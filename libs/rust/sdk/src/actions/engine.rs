use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use failure::ResultExt;
use humthreads::Builder;
use opentracingrust::Span;
use slog::debug;
use slog::trace;
use slog::warn;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_failure::SerializableFail;
use replicante_util_tracing::fail_span;
use replicante_util_upkeep::Upkeep;

use crate::actions::Action;
use crate::actions::ActionRecord;
use crate::actions::ActionState;
use crate::actions::ACTIONS;
use crate::metrics::ACTION_COUNT;
use crate::metrics::ACTION_DURATION;
use crate::metrics::ACTION_ERRORS;
use crate::metrics::ACTION_PRUNE_DURATION;
use crate::store::Transaction;
use crate::AgentContext;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Start background thread to execute registered actions.
pub fn spawn(context: AgentContext, upkeep: &mut Upkeep) -> Result<()> {
    let thread = Builder::new("r:b:actions")
        .full_name("replicante:base:actions:engine")
        .spawn(move |scope| {
            let logger = context.logger.clone();
            let execute_interval = Duration::from_secs(context.config.actions.execute_interval);
            let prune_interval = Duration::from_secs(context.config.actions.prune_interval);
            let engine = Engine::new(context);
            // Initialise last_prune to 2 * prune_interval ago to prune after start.
            let mut last_prune = Instant::now() - (2 * prune_interval);
            scope.activity("waiting to poll for actions");
            while !scope.should_shutdown() {
                let _activity = scope.scoped_activity("handling actions");
                if let Err(error) = engine.poll() {
                    capture_fail!(
                        &error,
                        logger,
                        "Error while processing an action";
                        failure_info(&error),
                    );
                }
                if last_prune.elapsed() > prune_interval {
                    last_prune = Instant::now();
                    let _activity = scope.scoped_activity("pruning actions history");
                    if let Err(error) = engine.clean() {
                        capture_fail!(
                            &error,
                            logger,
                            "Error while cleaning up historic actions";
                            failure_info(&error),
                        );
                    }
                }
                thread::sleep(execute_interval);
            }
        })
        .with_context(|_| ErrorKind::ThreadSpawn("actions engine"))?;
    upkeep.register_thread(thread);
    Ok(())
}

/// Actions engine logic.
struct Engine {
    context: AgentContext,
}

impl Engine {
    pub fn new(context: AgentContext) -> Engine {
        Engine { context }
    }

    /// Perform historic actions cleanup to prevent endless DB growth.
    pub fn clean(&self) -> Result<()> {
        trace!(self.context.logger, "Pruning actions history");
        let keep = self.context.config.actions.prune_keep;
        let limit = self.context.config.actions.prune_limit;
        let _timer = ACTION_PRUNE_DURATION.start_timer();
        self.context
            .store
            .with_transaction(|tx| tx.actions().prune(keep, limit, None))
    }

    /// Looks for running or pending actions and processes them.
    pub fn poll(&self) -> Result<()> {
        // Wrapped in `Some` to allow transition to optional Tracer easier.
        let mut span = Some(self.context.tracer.span("actions.poll").auto_finish());
        let rv = self.context.store.with_transaction(|tx| {
            let record = tx
                .action()
                .next(span.as_ref().map(|span| span.context().clone()))?;
            let record = match record {
                None => return Ok(()),
                Some(record) => record,
            };
            if let Some(span) = span.as_mut() {
                span.tag("action.kind", record.kind.clone());
                span.tag("action.id", record.id.to_string());
                match record.trace_get(&self.context.tracer) {
                    Ok(None) => (),
                    Ok(Some(context)) => span.follows(context),
                    Err(error) => {
                        capture_fail!(
                            &error,
                            self.context.logger,
                            "Failed to extract tracing context from action record";
                            failure_info(&error),
                            "id" => %&record.id,
                            "kind" => &record.kind,
                        );
                    }
                };
            }
            ACTION_COUNT.with_label_values(&[&record.kind]).inc();
            let action = match ACTIONS::get(&record.kind) {
                Some(action) => action,
                None => {
                    let error = ErrorKind::ActionNotAvailable(record.kind.clone());
                    return self.fail(tx, &record, error.into(), span.as_ref().map(Deref::deref));
                }
            };
            debug!(
                self.context.logger,
                "Invoking action handler";
                "id" => %&record.id,
                "kind" => &record.kind,
            );
            match self.call(tx, &record, action, span.as_mut().map(DerefMut::deref_mut)) {
                Err(error) => self.fail(tx, &record, error, span.as_ref().map(Deref::deref)),
                Ok(()) => Ok(()),
            }
        });
        match rv {
            Ok(()) => Ok(()),
            Err(error) => Err(fail_span(error, span.as_mut().map(DerefMut::deref_mut))),
        }
    }
}

impl Engine {
    fn call(
        &self,
        tx: &mut Transaction,
        record: &ActionRecord,
        action: Arc<dyn Action>,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let _timer = ACTION_DURATION
            .with_label_values(&[&record.kind])
            .start_timer();
        action.invoke(tx, record, span)
    }

    fn fail(
        &self,
        tx: &mut Transaction,
        record: &ActionRecord,
        error: Error,
        span: Option<&Span>,
    ) -> Result<()> {
        warn!(
            self.context.logger,
            "Action invocation failed";
            "id" => %&record.id,
            "kind" => &record.kind,
            failure_info(&error),
        );
        ACTION_ERRORS.with_label_values(&[&record.kind]).inc();
        let error = SerializableFail::from(&error);
        let error = serde_json::to_value(&error).with_context(|_| ErrorKind::ActionEncode)?;
        tx.action().transition(
            record,
            ActionState::Failed,
            error,
            span.map(|span| span.context().clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use replicante_util_failure::SerializableFail;

    use super::super::impls::debug::Progress;
    use super::Engine;
    use crate::actions::ActionRecord;
    use crate::actions::ActionRecordView;
    use crate::actions::ActionRequester;
    use crate::actions::ActionState;
    use crate::actions::ActionsRegister;
    use crate::actions::ACTIONS;
    use crate::AgentContext;

    #[test]
    fn fail_action_with_unkown_kind() {
        let action = ActionRecord::new("test", None, None, json!({}), ActionRequester::AgentApi);
        let id = action.id;
        let context = AgentContext::mock();
        context
            .store
            .with_transaction(|tx| tx.action().insert(action, None))
            .unwrap();
        let register = ActionsRegister::default();
        ACTIONS::test_with(register, || {
            let engine = Engine::new(context.clone());
            engine.poll().expect("poll failed to process action");
        });
        let action = context
            .store
            .with_transaction(|tx| tx.action().get(&id.to_string(), None))
            .unwrap()
            .unwrap();
        assert_eq!(id, action.id);
        assert_eq!(ActionState::Failed, *action.state());
        let payload = action
            .state_payload()
            .clone()
            .expect("need a state payload");
        let payload: SerializableFail = serde_json::from_value(payload).unwrap();
        assert_eq!(payload.error, "actions with kind test are not available");
    }

    #[test]
    fn no_action_noop() {
        let context = AgentContext::mock();
        let engine = Engine::new(context);
        engine.poll().expect("poll failed to process action");
    }

    #[test]
    fn transition_new_to_running() {
        let action = ActionRecord::new(
            "replicante.debug.progress".to_string(),
            None,
            None,
            json!({}),
            ActionRequester::AgentApi,
        );
        let id = action.id;
        let context = AgentContext::mock();
        context
            .store
            .with_transaction(|tx| tx.action().insert(action, None))
            .unwrap();
        let mut register = ActionsRegister::default();
        register.register_reserved(Progress {});
        ACTIONS::test_with(register, || {
            let engine = Engine::new(context.clone());
            engine.poll().expect("poll failed to process action");
        });
        let action = context
            .store
            .with_transaction(|tx| tx.action().get(&id.to_string(), None))
            .unwrap()
            .unwrap();
        assert_eq!(id, action.id);
        assert_eq!(ActionState::Running, *action.state());
    }
}
