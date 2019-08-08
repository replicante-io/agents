use slog::debug;
use slog::info;
use slog::warn;

use replicante_util_upkeep::Upkeep;

use crate::config::Agent as Config;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

mod actions_api;
#[cfg(any(debug_assertions, test))]
mod debug;
mod definition;
mod engine;
mod register;
#[cfg(test)]
mod tests;
pub mod utils;

pub use self::definition::Action;
pub use self::definition::ActionDescriptor;
pub use self::definition::ActionListItem;
pub use self::definition::ActionRecord;
pub use self::definition::ActionRequester;
pub use self::definition::ActionState;
pub use self::definition::ActionValidity;
pub use self::definition::ActionValidityError;
pub use self::register::ActionsRegister;
pub use self::register::ACTIONS;

/// Checks if agent actions are enabled.
///
///   * Agent actions are automatically enabled if `tls.clients_ca_bundle` is set.
///   * Agent actions can be explicitly disabled with the `actions.enabled` option.
///   * An error is returned if `actions.enabled` is `true` but `tls.clients_ca_bundle`
///     is not set.
pub fn actions_enabled(config: &Config) -> Result<bool> {
    if let Some(false) = config.actions.enabled {
        return Ok(false);
    }
    let mutual_tls = config
        .api
        .tls
        .as_ref()
        .map(|tls| tls.clients_ca_bundle.is_some())
        .unwrap_or(false);
    if !mutual_tls {
        if let Some(true) = config.actions.enabled {
            return Err(ErrorKind::ConfigClash(
                "can't enable actions without TLS client certificates",
            )
            .into());
        }
    }
    Ok(mutual_tls)
}

/// Initialise the actions system based on configuration.
pub fn initialise(context: &mut AgentContext, upkeep: &mut Upkeep) -> Result<()> {
    let enabled = actions_enabled(&context.config)?;
    if !enabled {
        warn!(context.logger, "Actions system not enabled");
        return Ok(());
    }

    debug!(context.logger, "Initialising actions system ...");
    let flags = context.config.api.trees.clone().into();
    context
        .api_addons
        .register(move |app, context| actions_api::configure_app(context, &flags, app));
    register_std_actions(context);
    ACTIONS::complete_registration();
    debug!(context.logger, "Actions registration phase completed");

    self::engine::spawn(context.clone(), upkeep)?;
    info!(context.logger, "Actions system initialised");
    Ok(())
}

/// Register standard agent actions.
fn register_std_actions(context: &AgentContext) {
    debug!(context.logger, "Registering standard actions");
    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
}
