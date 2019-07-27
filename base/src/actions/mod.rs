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
mod register;

pub use self::definition::Action;
pub use self::definition::ActionDescriptor;
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
pub fn initialise(context: &mut AgentContext, _upkeep: &mut Upkeep) -> Result<()> {
    let enabled = actions_enabled(&context.config)?;
    if !enabled {
        warn!(context.logger, "Actions system not enabled");
        return Ok(());
    }

    debug!(context.logger, "Initialising actions system ...");
    let flags = context.config.api.trees.clone().into();
    context
        .api_addons
        .register(move |app| actions_api::configure_app(&flags, app));
    register_std_actions(context);
    ACTIONS::complete_registration();
    // TODO: spawn actions engine thread.

    info!(context.logger, "Actions system initialised");
    Ok(())
}

/// Register standard agent actions.
fn register_std_actions(context: &AgentContext) {
    debug!(context.logger, "Registering standard actions");
    #[cfg(any(debug_assertions, test))]
    self::debug::register_debug_actions(context);
}

#[cfg(test)]
mod tests {
    use failure::Fail;

    use crate::config::Agent as Config;
    use crate::config::TlsConfig;

    #[test]
    fn disabled_by_default() {
        let config = Config::default();
        let enabled = super::actions_enabled(&config);
        assert!(!enabled.unwrap(), "actions should be disabled by default");
    }

    #[test]
    fn disabled_explicitly_with_tls() {
        let mut config = Config::default();
        let tls = TlsConfig {
            clients_ca_bundle: Some("clients".to_string()),
            server_cert: "server.crt".to_string(),
            server_key: "server.key".to_string(),
        };
        config.actions.enabled = Some(false);
        config.api.tls = Some(tls);
        let enabled = super::actions_enabled(&config);
        assert!(!enabled.unwrap(), "actions should be disabled by config");
    }

    #[test]
    fn enabled_implicitly_by_tls() {
        let mut config = Config::default();
        let tls = TlsConfig {
            clients_ca_bundle: Some("clients".to_string()),
            server_cert: "server.crt".to_string(),
            server_key: "server.key".to_string(),
        };
        config.api.tls = Some(tls);
        let enabled = super::actions_enabled(&config);
        assert!(
            enabled.unwrap(),
            "actions should be enabled by clients bundle",
        );
    }

    #[test]
    fn enabled_explicitly_without_tls() {
        let mut config = Config::default();
        config.actions.enabled = Some(true);
        match super::actions_enabled(&config) {
            Ok(_) => panic!("expected configuration error"),
            Err(error) => assert_eq!(error.name().unwrap(), "ConfigClash"),
        };
    }
}
