use std::collections::HashMap;
use std::collections::HashSet;

use slog::debug;
use slog::info;
use slog::warn;

use replicante_util_upkeep::Upkeep;

use crate::config::Agent as Config;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

mod definition;
mod engine;
mod impls;
mod register;
#[cfg(test)]
mod tests;
pub mod utils;

pub use self::definition::Action;
pub use self::definition::ActionDescriptor;
pub use self::definition::ActionListItem;
pub use self::definition::ActionRecord;
pub use self::definition::ActionRecordHistory;
pub use self::definition::ActionRequester;
pub use self::definition::ActionState;
pub use self::definition::ActionValidity;
pub use self::definition::ActionValidityError;
pub use self::register::ActionsRegister;
pub use self::register::ACTIONS;

lazy_static::lazy_static! {
    /// Codified version of the state transitions from docs/docs/assets/action-states.dot
    static ref ALLOWED_TRANSITIONS: HashMap<ActionState, HashSet<ActionState>> = {
        let mut transitions = HashMap::new();
        transitions.insert(ActionState::New, {
            let mut allowed = HashSet::new();
            allowed.insert(ActionState::Cancel);
            allowed.insert(ActionState::Done);
            allowed.insert(ActionState::Failed);
            allowed.insert(ActionState::Running);
            allowed
        });
        transitions.insert(ActionState::Cancel, {
            let mut allowed = HashSet::new();
            allowed.insert(ActionState::Cancelled);
            allowed.insert(ActionState::Failed);
            allowed
        });
        transitions.insert(ActionState::Running, {
            let mut allowed = HashSet::new();
            allowed.insert(ActionState::Done);
            allowed.insert(ActionState::Failed);
            allowed.insert(ActionState::Running);
            allowed
        });
        transitions
    };
}

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

/// Ensure the action state transition is allowed.
///
/// # Panics
/// If the state transition is not allowed this function panics.
pub fn ensure_transition_allowed(from: &ActionState, to: &ActionState) {
    let allowed = ALLOWED_TRANSITIONS
        .get(from)
        .map(|from| from.contains(to))
        .unwrap_or(false);
    if !allowed {
        panic!(
            "actions are not allowed to transition from {:?} to {:?}",
            from, to
        );
    }
}

/// Initialise the actions system based on configuration.
pub fn initialise(context: &mut AgentContext, upkeep: &mut Upkeep) -> Result<()> {
    let enabled = actions_enabled(&context.config)?;
    if !enabled {
        warn!(context.logger, "Actions system not enabled");
        return Ok(());
    }

    debug!(context.logger, "Initialising actions system ...");
    self::impls::register_std_actions(context);
    ACTIONS::complete_registration();
    debug!(context.logger, "Actions registration phase completed");

    self::engine::spawn(context.clone(), upkeep)?;
    info!(context.logger, "Actions system initialised");
    Ok(())
}
