use slog::debug;

use crate::Agent;
use crate::AgentContext;

#[cfg(any(debug_assertions, test))]
pub(crate) mod debug;
mod service;

/// Register standard agent actions.
pub fn register_std_actions(agent: &dyn Agent, context: &AgentContext) {
    debug!(context.logger, "Registering standard actions");
    self::service::register(agent, context);

    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
}
