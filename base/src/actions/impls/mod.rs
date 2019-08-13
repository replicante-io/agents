use slog::debug;

use crate::AgentContext;

#[cfg(any(debug_assertions, test))]
pub(crate) mod debug;

/// Register standard agent actions.
pub fn register_std_actions(context: &AgentContext) {
    debug!(context.logger, "Registering standard actions");
    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
}
