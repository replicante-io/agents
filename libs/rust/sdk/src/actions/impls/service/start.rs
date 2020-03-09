use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Span;
use serde_json::Value as Json;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::store::Transaction;
use crate::ErrorKind;
use crate::Result;

use super::supervisor::Supervisor;
use super::ServiceActionState;

// This is a minimum of 30 seconds, maybe it should become a configuration option.
const MAX_ATTEMPT_START: u8 = 30;

/// Stop the datastore service.
pub struct ServiceStart {
    supervisor: Arc<dyn Supervisor>,
}

impl ServiceStart {
    pub fn new(supervisor: &Arc<dyn Supervisor>) -> ServiceStart {
        let supervisor = Arc::clone(supervisor);
        ServiceStart { supervisor }
    }
}

impl Action for ServiceStart {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.io/service.start".into(),
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
