use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_logging::Config as LoggingConfig;
use replicante_logging::LoggingLevel;
use replicante_util_tracing::Config as TracerConfig;

mod actions;
mod api;
mod sentry;

pub use self::actions::ActionsConfig;
pub use self::api::APIConfig;
pub use self::api::TlsConfig;
pub use self::sentry::SentryCaptureApi;
pub use self::sentry::SentryConfig;

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

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Sentry integration configuration
    #[serde(default)]
    pub sentry: Option<SentryConfig>,

    /// Enable the update checker (optional).
    #[serde(default = "Agent::default_update_checker")]
    pub update_checker: bool,

    /// OpenTracing configuration
    #[serde(default)]
    pub tracing: TracerConfig,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            actions: ActionsConfig::default(),
            api: APIConfig::default(),
            cluster_display_name_override: None,
            logging: LoggingConfig::default(),
            sentry: None,
            update_checker: false,
            tracing: TracerConfig::default(),
        }
    }
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
}

#[cfg(test)]
mod tests {
    use super::APIConfig;
    use super::Agent;

    #[test]
    fn override_defauts() {
        APIConfig::set_default_bind(String::from("1.2.3.4:5678"));
        let agent = Agent::default();
        assert_eq!(agent.api.bind, "1.2.3.4:5678");
    }
}
