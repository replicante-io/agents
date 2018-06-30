use replicante_util_tracing::Config as TracerConfig;

mod api;
pub use self::api::APIConfig;


/// Stores the base agent configuration options.
///
/// Configuration options used by the base agent utilities and structs.
/// Attributes are public to make it easier to use configuration values
/// but are not meant to be changed after the configuration is finialised.
///
/// New configuration values are created with `AgentConfig::default` and
/// changing the attributes as desired.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// API server configuration
    #[serde(default)]
    pub api: APIConfig,

    /// OpenTracing configuration
    #[serde(default)]
    pub tracer: TracerConfig,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            api: APIConfig::default(),
            tracer: TracerConfig::default(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Agent;
    use super::APIConfig;

    #[test]
    fn override_defauts() {
        APIConfig::set_default_bind(String::from("1.2.3.4:5678"));
        let agent = Agent::default();
        assert_eq!(agent.api.bind, "1.2.3.4:5678");
    }
}
