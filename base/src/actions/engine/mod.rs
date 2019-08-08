use std::sync::Arc;
use std::thread;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_failure::SerializableFail;
use replicante_util_tracing::fail_span;
use replicante_util_upkeep::Upkeep;

#[cfg(test)]
mod tests;

use crate::actions::Action;
use crate::actions::ActionRecord;
use crate::actions::ActionState;
use crate::actions::ACTIONS;
use crate::metrics::ACTION_COUNT;
use crate::metrics::ACTION_DURATION;
use crate::metrics::ACTION_ERRORS;
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
            let engine = Engine::new(context);
            scope.activity("waiting to poll for actions");
            while !scope.should_shutdown() {
                if let Err(error) = engine.poll() {
                    capture_fail!(
                        &error,
                        logger,
                        "Error while processing an action";
                        failure_info(&error),
                    );
                }
                thread::sleep(Duration::from_secs(1));
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
                span.tag("action.kind", record.action.clone());
                span.tag("action.id", record.id.to_string());
            }
            ACTION_COUNT.with_label_values(&[&record.action]).inc();
            let action = match ACTIONS::get(&record.action) {
                Some(action) => action,
                None => {
                    let error = ErrorKind::ActionNotAvailable(record.action.clone());
                    return self.fail(tx, &record, error.into());
                }
            };
            match self.call(tx, &record, action) {
                Err(error) => self.fail(tx, &record, error),
                Ok(()) => Ok(()),
            }
        });
        match rv {
            Ok(()) => Ok(()),
            Err(error) => match span.as_mut() {
                None => Err(error),
                Some(span) => Err(fail_span(error, span)),
            },
        }
    }
}

impl Engine {
    fn call(
        &self,
        tx: &mut Transaction,
        record: &ActionRecord,
        action: Arc<dyn Action>,
    ) -> Result<()> {
        let _timer = ACTION_DURATION
            .with_label_values(&[&record.action])
            .start_timer();
        action.invoke(tx, record)
    }

    fn fail(&self, tx: &mut Transaction, record: &ActionRecord, error: Error) -> Result<()> {
        ACTION_ERRORS.with_label_values(&[&record.action]).inc();
        let error = SerializableFail::from(&error);
        let error = serde_json::to_value(&error).with_context(|_| ErrorKind::ActionEncode)?;
        tx.action()
            .transition(record, ActionState::Failed, error, None)
    }
}
