use opentracingrust::Span;
use serde_json::Value as Json;
use slog::debug;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::actions::ACTIONS;
use crate::store::Transaction;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

/// Register debugging actions.
pub fn register_debug_actions(context: &AgentContext) {
    debug!(context.logger, "Registering debug actions");
    ACTIONS::register_reserved(Fail {});
    ACTIONS::register_reserved(Progress {});
    ACTIONS::register_reserved(Success {});
}

/// Debugging action that always fails.
pub(crate) struct Fail {}

impl Action for Fail {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "agent.replicante.io/debug.fail".into(),
            description: "Debugging action that always fails".into(),
        }
    }

    fn invoke(
        &self,
        _: &mut Transaction,
        _: &dyn ActionRecordView,
        _: Option<&mut Span>,
    ) -> Result<()> {
        let error = "triggered debugging action that fails".into();
        Err(ErrorKind::FreeForm(error).into())
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Debugging action that progresses over time.
pub(crate) struct Progress {}

impl Action for Progress {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "agent.replicante.io/debug.progress".into(),
            description: "Debugging action that progresses over time".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let next_state = if *record.state() == ActionState::New {
            ActionState::Running
        } else {
            ActionState::Done
        };
        tx.action().transition(
            record,
            next_state,
            None,
            span.map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Debugging action that always succeed.
pub(crate) struct Success {}

impl Action for Success {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "agent.replicante.io/debug.success".into(),
            description: "Debugging action that always succeed".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        tx.action().transition(
            record,
            ActionState::Done,
            None,
            span.map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}
