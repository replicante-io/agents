use serde_json::Value as Json;
use slog::debug;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecord;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::actions::ACTIONS;
use crate::store::Transaction;
use crate::AgentContext;
use crate::Result;

/// Register debugging actions.
pub fn register_debug_actions(context: &AgentContext) {
    debug!(context.logger, "Registering debug actions");
    ACTIONS::register_reserved(Fail {});
    ACTIONS::register_reserved(Progress {});
    ACTIONS::register_reserved(Success {});
}

/// Debugging action that always fails.
pub(super) struct Fail {}

impl Action for Fail {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.debug.fail".into(),
            description: "Debugging action that always fails".into(),
        }
    }

    fn invoke(&self, _: &mut Transaction, _: &ActionRecord) -> Result<()> {
        panic!("TODO: Fail::invoke")
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Debugging action that progresses over time.
pub(super) struct Progress {}

impl Action for Progress {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.debug.progress".into(),
            description: "Debugging action that progresses over time".into(),
        }
    }

    fn invoke(&self, tx: &mut Transaction, record: &ActionRecord) -> Result<()> {
        // TODO: when added, go to success if ! new.
        let next_state = ActionState::Running;
        //let next_state = if record.state == ActionState::New {
        //    ActionState::Running
        //} else {
        //    ActionState::Running
        //};
        tx.action().transition(record, next_state, None, None)
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

/// Debugging action that always succeed.
pub(super) struct Success {}

impl Action for Success {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.debug.success".into(),
            description: "Debugging action that always succeed".into(),
        }
    }

    fn invoke(&self, _: &mut Transaction, _: &ActionRecord) -> Result<()> {
        panic!("TODO: Success::invoke")
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}
