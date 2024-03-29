use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use replicante_logging::Config as LoggingConfig;
use replicante_logging::LoggingLevel;
use replicante_util_tracing::Config as TracerConfig;

mod actions;
mod api;
mod sentry;
mod service;

pub use self::actions::ActionsConfig;
pub use self::actions::ExternalActionConfig;
pub use self::api::APIConfig;
pub use self::api::TlsConfig;
pub use self::sentry::SentryConfig;
pub use self::service::ServiceConfig;

/// Stores the base agent configuration options.
///
/// Configuration options used by the base agent utilities and structs.
/// Attributes are public to make it easier to use configuration values
/// but are not meant to be changed after the configuration is finialised.
///
/// New configuration values are created with `AgentConfig::default` and
/// changing the attributes as desired.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// Actions configuration
    #[serde(default)]
    pub actions: ActionsConfig,

    /// API server configuration
    #[serde(default)]
    pub api: APIConfig,

    /// Override the cluster display name, or set it if none was detected.
    #[serde(default)]
    pub cluster_display_name_override: Option<String>,

    /// Location for the agent to store persistent data.
    pub db: String,

    /// User defined external actions.
    #[serde(default)]
    pub external_actions: BTreeMap<String, ExternalActionConfig>,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Sentry integration configuration.
    #[serde(default)]
    pub sentry: Option<SentryConfig>,

    /// Service supervisor configuration.
    #[serde(default)]
    pub service: Option<ServiceConfig>,

    /// OpenTracing configuration.
    #[serde(default)]
    pub tracing: TracerConfig,

    /// Enable the update checker (optional).
    #[serde(default = "Agent::default_update_checker")]
    pub update_checker: bool,
}

impl Agent {
    fn default_update_checker() -> bool {
        false
    }

    /// Apply transformations to the configuration to derive some parameters.
    ///
    /// Transformations:
    ///
    ///   * Apply verbose debug level logic.
    pub fn transform(mut self) -> Self {
        if self.logging.level == LoggingLevel::Debug && !self.logging.verbose {
            self.logging.level = LoggingLevel::Info;
            self.logging
                .modules
                .entry("replicante".into())
                .or_insert(LoggingLevel::Debug);
        }
        self
    }

    /// Mock an agent configuration.
    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock() -> Self {
        Agent {
            actions: ActionsConfig::default(),
            api: APIConfig::default(),
            cluster_display_name_override: None,
            db: "mock.db".into(),
            external_actions: BTreeMap::default(),
            logging: LoggingConfig::default(),
            sentry: None,
            service: None,
            tracing: TracerConfig::default(),
            update_checker: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::APIConfig;
    use super::Agent;

    #[test]
    fn override_defauts() {
        APIConfig::set_default_bind(String::from("1.2.3.4:5678"));
        let agent = Agent::mock();
        assert_eq!(agent.api.bind, "1.2.3.4:5678");
    }
}
