use std::collections::HashMap;
use std::sync::Arc;

use slog::debug;

use crate::actions::Action;
use crate::actions::ActionHook;
use crate::AgentContext;
use crate::Result;

#[cfg(any(debug_assertions, test))]
pub(crate) mod debug;
mod external;
mod service;
mod test;

/// Register standard agent actions.
pub fn register_std_actions(
    context: &AgentContext,
    hooks: HashMap<ActionHook, Arc<dyn Action>>,
) -> Result<()> {
    debug!(context.logger, "Registering standard actions");
    let graceful = hooks.get(&ActionHook::StoreGracefulStop).cloned();
    self::external::register(context)?;
    self::service::register(context, graceful);
    self::test::register(context);

    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
    Ok(())
}
