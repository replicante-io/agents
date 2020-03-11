use slog::debug;

use crate::actions::ACTIONS;
use crate::AgentContext;

mod ping;

/// Register all test actions.
pub fn register(context: &AgentContext) {
    debug!(context.logger, "Registering test actions");
    ACTIONS::register_reserved(self::ping::Ping {});
}
