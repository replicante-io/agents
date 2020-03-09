use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Span;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value as Json;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecord;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::actions::ActionValidityError;
use crate::store::Transaction;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

lazy_static::lazy_static! {
    static ref DEFAULT_ARG_NULL: Json = json!(null);
    static ref DEFAULT_ARG_OBJECT: Json = json!({});
}

/// An action to run with the "scope" its arguments are received under.
struct ActionScope {
    action: ActionScopeWrapper,
    scope: &'static str,
}

impl ActionScope {
    /// Grab the arguments to pass to this action.
    fn args<'a>(&self, args: &'a Json) -> &'a Json {
        if args.is_null() {
            return &DEFAULT_ARG_NULL;
        }
        match args.get(self.scope) {
            None => &DEFAULT_ARG_OBJECT,
            Some(args) => args,
        }
    }
}

/// Wrap dynamic `Action`s into either an `Arc` or a `Box`.
enum ActionScopeWrapper {
    Arc(Arc<dyn Action>),
    Box(Box<dyn Action>),
}

impl ActionScopeWrapper {
    /// Invoke the wrapped action.
    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        match self {
            ActionScopeWrapper::Arc(ref action) => action.invoke(tx, record, span),
            ActionScopeWrapper::Box(ref action) => action.invoke(tx, record, span),
        }
    }

    /// Validate args agains the wrapped action.
    fn validate_args(&self, args: &Json) -> ActionValidity {
        match self {
            ActionScopeWrapper::Arc(ref action) => action.validate_args(args),
            ActionScopeWrapper::Box(ref action) => action.validate_args(args),
        }
    }
}

/// Execute sub-actions sequencially.
///
/// The action fails as soon as any sub-action fails.
/// Any number of actions can be registered with an `AndThen` but at least once must be provided.
pub struct AndThen {
    descriptor: ActionDescriptor,
    stages: Vec<ActionScope>,
}

impl AndThen {
    /// Build a new `AndThen` action.
    pub fn build() -> AndThenBuilder {
        AndThenBuilder {
            descriptor: None,
            stages: Vec::new(),
        }
    }
}

impl Action for AndThen {
    fn describe(&self) -> ActionDescriptor {
        self.descriptor.clone()
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        // If the action is finished do nothing.
        // We should never be called is the action is finished but
        // having this check here means we can make more assumptions later.
        if record.state().is_finished() {
            return Ok(());
        }

        // Fetch the current action stage and stage, or start anew.
        // If the current stage is finished, start the next.
        let mut state: AndThenState =
            ActionRecordView::structured_state_payload(record)?.unwrap_or_default();
        if state.state == ActionState::Done {
            let stage = state.stage + 1;
            state = AndThenState::default();
            state.stage = stage;
        }

        // Invoke the correct action.
        let stage = match self.stages.get(state.stage) {
            None => {
                return Err(ErrorKind::FreeForm(
                    "can't find action for the current stage".into(),
                ))
                .with_context(|_| ErrorKind::ActionDecode)
                .map_err(Error::from)
            }
            Some(stage) => stage,
        };
        let args = stage.args(record.args());
        let more_stages = state.stage < self.stages.len() - 1;
        let view = AndThenRecord {
            args,
            more_stages,
            record,
            state,
        };
        stage.action.invoke(tx, &view, span)
    }

    fn validate_args(&self, args: &Json) -> ActionValidity {
        if !(args.is_null() || args.is_object()) {
            return Err(ActionValidityError::InvalidArgs(
                "expected null or object".into(),
            ));
        }
        for stage in &self.stages {
            let stage_args = stage.args(args);
            stage.action.validate_args(stage_args)?;
        }
        Ok(())
    }
}

/// `AndThen` sequential actions builder.
pub struct AndThenBuilder {
    descriptor: Option<ActionDescriptor>,
    stages: Vec<ActionScope>,
}

impl AndThenBuilder {
    /// Set the `ActionDescriptor` for the finished action.
    pub fn describe(mut self, descriptor: ActionDescriptor) -> Self {
        self.descriptor = Some(descriptor);
        self
    }

    /// Consume the builder and returns an `AndThen` action.
    ///
    /// # Panics
    ///
    ///   * If the `ActionDescriptor` is not defined (call `AndThenBuilder::describe`).
    ///   * If `Action`s to execute are not defined (call `AndThenBuilder::and_then` at least once).
    pub fn finish(self) -> AndThen {
        let stages = self.stages;
        if stages.is_empty() {
            panic!("call AndThenBuilder::and_then to register at least one action");
        }
        let descriptor = self
            .descriptor
            .expect("action descriptor must be set, use AndThenBuilder::describe");
        AndThen { descriptor, stages }
    }

    /// Append an action to the execution sequence.
    ///
    /// Arguments passed to the actions are `scope`d in the root action's arguments.
    /// For example:
    ///
    ///   * With `{"some-scope": 3}` as the root arguments.
    ///   * An action scoped under `some-scope` will receive `3` as its arguments.
    pub fn and_then<A>(mut self, action: A, scope: &'static str) -> Self
    where
        A: Action,
    {
        let action = ActionScopeWrapper::Box(Box::new(action));
        self.stages.push(ActionScope { action, scope });
        self
    }

    /// Same as `AndThen::and_then` but for actions wrapped by and `Arc`.
    pub(crate) fn and_then_arc(mut self, action: Arc<dyn Action>, scope: &'static str) -> Self {
        let action = ActionScopeWrapper::Arc(action);
        self.stages.push(ActionScope { action, scope });
        self
    }
}

/// `ActionRecordView` to proxy action state and property access from sub-actions.
struct AndThenRecord<'a> {
    args: &'a Json,
    more_stages: bool,
    record: &'a dyn ActionRecordView,
    state: AndThenState,
}

impl<'a> ActionRecordView for AndThenRecord<'a> {
    fn args(&self) -> &Json {
        self.args
    }

    fn inner(&self) -> &ActionRecord {
        self.record.inner()
    }

    fn map_transition(
        &self,
        transition_to: ActionState,
        payload: Option<Json>,
    ) -> Result<(ActionState, Option<Json>)> {
        let mut state = self.state.clone();
        state.payload = payload;
        state.state = transition_to.clone();
        let transition_to = match transition_to {
            ActionState::Done if self.more_stages => ActionState::Running,
            transition_to => transition_to,
        };
        let payload = serde_json::to_value(state).with_context(|_| ErrorKind::ActionEncode)?;
        Ok((transition_to, Some(payload)))
    }

    fn state(&self) -> &ActionState {
        &self.state.state
    }

    fn state_payload(&self) -> &Option<Json> {
        &self.state.payload
    }
}

/// Persisted state of the composite `AndThen` action.
#[derive(Clone, Serialize, Deserialize)]
struct AndThenState {
    payload: Option<Json>,
    stage: usize,
    state: ActionState,
}

impl Default for AndThenState {
    fn default() -> Self {
        AndThenState {
            payload: None,
            stage: 0,
            state: ActionState::New,
        }
    }
}

#[cfg(test)]
mod tests {
    use opentracingrust::Span;
    use serde_json::json;
    use serde_json::Value as Json;

    use super::AndThen;
    use crate::actions::impls::debug::Progress;
    use crate::actions::impls::debug::Success;
    use crate::actions::Action;
    use crate::actions::ActionDescriptor;
    use crate::actions::ActionRecord;
    use crate::actions::ActionRecordView;
    use crate::actions::ActionRequester;
    use crate::actions::ActionState;
    use crate::actions::ActionValidity;
    use crate::actions::ActionValidityError;
    use crate::store::Store;
    use crate::store::Transaction;
    use crate::Result;

    struct ExpectIntArg {}
    impl Action for ExpectIntArg {
        fn describe(&self) -> ActionDescriptor {
            panic!("method not needed for tests")
        }

        fn invoke(
            &self,
            _: &mut Transaction,
            _: &dyn ActionRecordView,
            _: Option<&mut Span>,
        ) -> Result<()> {
            panic!("method not needed for tests")
        }

        fn validate_args(&self, args: &Json) -> ActionValidity {
            match args.is_number() {
                false => Err(ActionValidityError::InvalidArgs("not a number".into())),
                true => Ok(()),
            }
        }
    }

    struct Fail {}
    impl Action for Fail {
        fn describe(&self) -> ActionDescriptor {
            ActionDescriptor {
                kind: "test.replicante.io/action2".into(),
                description: "Replicante test action 2".into(),
            }
        }

        fn invoke(
            &self,
            tx: &mut Transaction,
            record: &dyn ActionRecordView,
            _: Option<&mut Span>,
        ) -> Result<()> {
            tx.action()
                .transition(record, ActionState::Failed, Json::from(21), None)
        }

        fn validate_args(&self, _: &Json) -> ActionValidity {
            Ok(())
        }
    }

    /// Helper function to create a new clean record.
    fn mkrecord(action: &dyn Action) -> ActionRecord {
        let kind = action.describe().kind;
        ActionRecord::new(kind, None, None, json!(null), ActionRequester::AgentApi)
    }

    #[test]
    fn build_action() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor.clone())
            .and_then(Success {}, "action_one")
            .and_then(Success {}, "action_two")
            .finish();
        assert_eq!(descriptor, action.describe());
    }

    #[test]
    #[should_panic(expected = "call AndThenBuilder::and_then to register at least one action")]
    fn build_action_empty_panics() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let _ = AndThen::build().describe(descriptor).finish();
    }

    #[test]
    #[should_panic(expected = "action descriptor must be set, use AndThenBuilder::describe")]
    fn build_action_no_description_panics() {
        let _ = AndThen::build().and_then(Success {}, "action_one").finish();
    }

    #[test]
    fn invoke_action_new() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor)
            .and_then(Progress {}, "action_one")
            .finish();
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let record = mkrecord(&action);
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Running);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "payload": null,
                        "stage": 0,
                        "state": "RUNNING",
                    })
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn invoke_action_running() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor)
            .and_then(Progress {}, "action_one")
            .and_then(Progress {}, "action_two")
            .finish();
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let mut record = mkrecord(&action);
                record.set_state(ActionState::Running);
                record.set_state_payload(Some(json!({
                    "payload": null,
                    "stage": 0,
                    "state": "RUNNING",
                })));
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Running);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "payload": null,
                        "stage": 0,
                        "state": "DONE",
                    })
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn invoke_action_done() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor)
            .and_then(Progress {}, "action_one")
            .finish();
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let mut record = mkrecord(&action);
                record.set_state(ActionState::Running);
                record.set_state_payload(Some(json!({
                    "payload": null,
                    "stage": 0,
                    "state": "RUNNING",
                })));
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Done);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "payload": null,
                        "stage": 0,
                        "state": "DONE",
                    })
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn start_next_action() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor)
            .and_then(Progress {}, "action_one")
            .and_then(Progress {}, "action_two")
            .finish();
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let mut record = mkrecord(&action);
                record.set_state(ActionState::Running);
                record.set_state_payload(Some(json!({
                    "payload": null,
                    "stage": 0,
                    "state": "DONE",
                })));
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Running);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "payload": null,
                        "stage": 1,
                        "state": "RUNNING",
                    })
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn skip_next_action_after_fail() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor)
            .and_then(Fail {}, "action_one")
            .and_then(Progress {}, "action_two")
            .finish();
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let record = mkrecord(&action);
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Failed);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "payload": 21,
                        "stage": 0,
                        "state": "FAILED",
                    })
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn validite_scoped_args_fail() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor.clone())
            .and_then(ExpectIntArg {}, "int_arg1")
            .and_then(ExpectIntArg {}, "int_arg2")
            .finish();
        let args = json!({ "int_arg1": 3 });
        match action.validate_args(&args) {
            Ok(()) => panic!("expected validation to fail"),
            Err(ActionValidityError::InvalidArgs(msg)) => assert_eq!("not a number", msg),
        }
    }

    #[test]
    fn validite_scoped_args_fail_type() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor.clone())
            .and_then(ExpectIntArg {}, "int_arg1")
            .and_then(ExpectIntArg {}, "int_arg2")
            .finish();
        let args = json!("abc");
        match action.validate_args(&args) {
            Ok(()) => panic!("expected validation to fail"),
            Err(ActionValidityError::InvalidArgs(msg)) => {
                assert_eq!("expected null or object", msg);
            }
        }
    }

    #[test]
    fn validite_scoped_args_pass() {
        let descriptor = ActionDescriptor {
            kind: "test.replicante.io/some.composed.action".into(),
            description: "Perform sequential actions".into(),
        };
        let action = AndThen::build()
            .describe(descriptor.clone())
            .and_then(ExpectIntArg {}, "int_arg1")
            .and_then(ExpectIntArg {}, "int_arg2")
            .finish();
        let args = json!({
            "int_arg1": 3,
            "int_arg2": 33,
        });
        action.validate_args(&args).unwrap();
    }
}
