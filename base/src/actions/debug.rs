use slog::debug;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ACTIONS;
use crate::AgentContext;

/// Register debugging actions.
pub fn register_debug_actions(context: &AgentContext) {
    debug!(context.logger, "Registering debug actions");
    ACTIONS::register_reserved(Fail {});
    ACTIONS::register_reserved(Progress {});
    ACTIONS::register_reserved(Success {});
}

/// Debugging action that always fails.
struct Fail {}

impl Action for Fail {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            id: "replicante.debug.fail".into(),
            description: "Debugging action that always fails".into(),
        }
    }
}

/// Debugging action that progresses over time.
struct Progress {}

impl Action for Progress {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            id: "replicante.debug.process".into(),
            description: "Debugging action that progresses over time".into(),
        }
    }
}

/// Debugging action that always succeed.
struct Success {}

impl Action for Success {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            id: "replicante.debug.succeed".into(),
            description: "Debugging action that always succeed".into(),
        }
    }
}
