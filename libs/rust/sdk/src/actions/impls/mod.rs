use std::collections::HashMap;
use std::sync::Arc;

use slog::debug;

use crate::actions::Action;
use crate::actions::ActionHook;
use crate::Agent;
use crate::AgentContext;

#[cfg(any(debug_assertions, test))]
pub(crate) mod debug;
mod service;

/// Register standard agent actions.
pub fn register_std_actions(
    agent: &dyn Agent,
    context: &AgentContext,
    hooks: HashMap<ActionHook, Arc<dyn Action>>,
) {
    debug!(context.logger, "Registering standard actions");
    let graceful = hooks.get(&ActionHook::StoreGracefulStop).cloned();
    self::service::register(agent, context, graceful);

    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
}
