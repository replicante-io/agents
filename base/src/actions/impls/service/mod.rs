use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Span;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecord;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::actions::ACTIONS;
use crate::store::Transaction;
use crate::Agent;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

mod supervisor;

use self::supervisor::Supervisor;

// This is a minimum of 30 seconds, maybe it should become a configuration option.
const MAX_ATTEMPT_START: u8 = 30;
const MAX_ATTEMPT_STOP: u8 = 30;

/// Register all service related actions.
pub fn register(agent: &dyn Agent, context: &AgentContext) {
    let supervisor = self::supervisor::factory(agent, context);
    ACTIONS::register_reserved(ServiceRestart {
        start: ServiceStart {
            supervisor: supervisor.clone(),
        },
        stop: ServiceStop {
            supervisor: supervisor.clone(),
        },
    });
    ACTIONS::register_reserved(ServiceStart {
        supervisor: supervisor.clone(),
    });
    ACTIONS::register_reserved(ServiceStop {
        supervisor: supervisor.clone(),
    });
}

/// Stop the datastore service, then start it if possible.
struct ServiceRestart {
    start: ServiceStart,
    stop: ServiceStop,
}

impl Action for ServiceRestart {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.service.restart".into(),
            description: "Stop/Start the datstore service".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let state: ServiceRestartState =
            ActionRecordView::structured_state_payload(record)?.unwrap_or_default();
        match state {
            ServiceRestartState::Done => Ok(()),
            ServiceRestartState::NotInitialised => {
                let state = ServiceRestartState::Stopping(ActionState::New, None);
                let view = ServiceRestartView { record, state };
                self.stop.invoke(tx, &view, span)
            }
            ServiceRestartState::Stopping(state, payload) => {
                let state = ServiceRestartState::Stopping(state, payload);
                let view = ServiceRestartView { record, state };
                self.stop.invoke(tx, &view, span)
            }
            ServiceRestartState::Starting(state, payload) => {
                let state = ServiceRestartState::Starting(state, payload);
                let view = ServiceRestartView { record, state };
                self.start.invoke(tx, &view, span)
            }
        }
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Persisted composite state of a service restart action.
#[derive(Serialize, Deserialize)]
enum ServiceRestartState {
    NotInitialised,
    Stopping(ActionState, Option<Json>),
    Starting(ActionState, Option<Json>),
    Done,
}

impl Default for ServiceRestartState {
    fn default() -> ServiceRestartState {
        ServiceRestartState::NotInitialised
    }
}

/// ServiceRestartState view for start and stop actions.
struct ServiceRestartView<'a> {
    record: &'a dyn ActionRecordView,
    state: ServiceRestartState,
}

impl<'a> ActionRecordView for ServiceRestartView<'a> {
    fn inner(&self) -> &ActionRecord {
        self.record.inner()
    }

    fn map_transition(
        &self,
        transition_to: ActionState,
        payload: Option<Json>,
    ) -> Result<(ActionState, Option<Json>)> {
        let (transition_to, payload) = self.record.map_transition(transition_to, payload)?;
        match &self.state {
            ServiceRestartState::Stopping(_, _) if transition_to == ActionState::Done => {
                let transition_to = ActionState::Running;
                let payload = ServiceRestartState::Starting(ActionState::New, None);
                let payload =
                    serde_json::to_value(payload).with_context(|_| ErrorKind::ActionEncode)?;
                Ok((transition_to, Some(payload)))
            }
            ServiceRestartState::Stopping(_, _) => {
                let payload = ServiceRestartState::Stopping(transition_to.clone(), payload);
                let payload =
                    serde_json::to_value(payload).with_context(|_| ErrorKind::ActionEncode)?;
                Ok((transition_to, Some(payload)))
            }
            ServiceRestartState::Starting(_, _) if transition_to == ActionState::Done => {
                let transition_to = ActionState::Done;
                let payload = ServiceRestartState::Done;
                let payload =
                    serde_json::to_value(payload).with_context(|_| ErrorKind::ActionEncode)?;
                Ok((transition_to, Some(payload)))
            }
            ServiceRestartState::Starting(_, _) => {
                let payload = ServiceRestartState::Starting(transition_to.clone(), payload);
                let payload =
                    serde_json::to_value(payload).with_context(|_| ErrorKind::ActionEncode)?;
                Ok((transition_to, Some(payload)))
            }
            _ => panic!("ServiceRestartView state is not starting or stopping"),
        }
    }

    fn state(&self) -> &ActionState {
        match &self.state {
            ServiceRestartState::Stopping(ref state, _) => state,
            ServiceRestartState::Starting(ref state, _) => state,
            _ => panic!("ServiceRestartView state is not starting or stopping"),
        }
    }

    fn state_payload(&self) -> &Option<Json> {
        match &self.state {
            ServiceRestartState::Stopping(_, ref payload) => payload,
            ServiceRestartState::Starting(_, ref payload) => payload,
            _ => panic!("ServiceRestartView state is not starting or stopping"),
        }
    }
}

/// Stop the datastore service.
struct ServiceStart {
    supervisor: Arc<dyn Supervisor>,
}

impl Action for ServiceStart {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.service.start".into(),
            description: "Start the datstore service".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let mut progress: ServiceActionState =
            ActionRecordView::structured_state_payload(record)?.unwrap_or_default();

        // If the action is new attempt to start the service.
        if *record.state() == ActionState::New {
            self.supervisor.start()?;
        }

        // Check if the service is running.
        let pid = self.supervisor.pid()?;
        progress.pid = pid;
        if progress.pid.is_some() {
            progress.message = Some("the service is running".into());
            let payload =
                serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
            return tx.action().transition(
                record,
                ActionState::Done,
                payload,
                span.as_ref().map(|span| span.context().clone()),
            );
        }

        // If we have been waiting too long fail.
        if progress.attempt >= MAX_ATTEMPT_START {
            progress.message = Some("the service did not start in time".into());
            let payload =
                serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
            return tx.action().transition(
                record,
                ActionState::Failed,
                payload,
                span.as_ref().map(|span| span.context().clone()),
            );
        }

        // Service still running, record attempt and wait.
        progress.attempt += 1;
        let payload = serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
        tx.action().transition(
            record,
            ActionState::Running,
            payload,
            span.as_ref().map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Stop the datastore service.
struct ServiceStop {
    supervisor: Arc<dyn Supervisor>,
}

impl Action for ServiceStop {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.service.stop".into(),
            description: "Stop the datstore service".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let mut progress: ServiceActionState =
            ActionRecordView::structured_state_payload(record)?.unwrap_or_default();

        // If the action is new attempt to stop the service.
        if *record.state() == ActionState::New {
            self.supervisor.stop()?;
        }

        // Check if the service is running.
        let pid = self.supervisor.pid()?;
        progress.pid = pid;
        if progress.pid.is_none() {
            progress.message = Some("the service is not running".into());
            let payload =
                serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
            return tx.action().transition(
                record,
                ActionState::Done,
                payload,
                span.as_ref().map(|span| span.context().clone()),
            );
        }

        // If we have been waiting too long fail.
        if progress.attempt >= MAX_ATTEMPT_STOP {
            progress.message = Some("the service did not stop in time".into());
            let payload =
                serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
            return tx.action().transition(
                record,
                ActionState::Failed,
                payload,
                span.as_ref().map(|span| span.context().clone()),
            );
        }

        // Service still running, record attempt and wait.
        progress.attempt += 1;
        let payload = serde_json::to_value(progress).with_context(|_| ErrorKind::ActionEncode)?;
        tx.action().transition(
            record,
            ActionState::Running,
            payload,
            span.as_ref().map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Persisted progress of service start/stop actions.
#[derive(Serialize, Deserialize)]
struct ServiceActionState {
    attempt: u8,
    message: Option<String>,
    pid: Option<String>,
}

impl Default for ServiceActionState {
    fn default() -> Self {
        ServiceActionState {
            attempt: 0,
            message: None,
            pid: None,
        }
    }
}
