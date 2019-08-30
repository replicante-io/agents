use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Span;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;

use crate::actions::advanced::AndThen;
use crate::actions::Action;
use crate::actions::ActionDescriptor;
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
    let restart = AndThen::build()
        .describe(ActionDescriptor {
            kind: "replicante.service.restart".into(),
            description: "Stop/Start the datstore service".into(),
        })
        .and_then(ServiceStop::new(&supervisor), "stop")
        .and_then(ServiceStart::new(&supervisor), "start")
        .finish();
    ACTIONS::register_reserved(ServiceStart::new(&supervisor));
    ACTIONS::register_reserved(ServiceStop::new(&supervisor));
    ACTIONS::register_reserved(restart);
}

/// Stop the datastore service.
struct ServiceStart {
    supervisor: Arc<dyn Supervisor>,
}

impl ServiceStart {
    fn new(supervisor: &Arc<dyn Supervisor>) -> ServiceStart {
        let supervisor = Arc::clone(supervisor);
        ServiceStart { supervisor }
    }
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

impl ServiceStop {
    fn new(supervisor: &Arc<dyn Supervisor>) -> ServiceStop {
        let supervisor = Arc::clone(supervisor);
        ServiceStop { supervisor }
    }
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
