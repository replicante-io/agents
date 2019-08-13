use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Actions configuration
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionsConfig {
    /// Enable/disable agent actions.
    #[serde(default)]
    pub enabled: Option<bool>,

    /// Delay, in seconds, between action executions.
    #[serde(default = "ActionsConfig::default_execute_interval")]
    pub execute_interval: u64,

    /// Delay, in seconds, between historical action prune cycles.
    #[serde(default = "ActionsConfig::default_prune_interval")]
    pub prune_interval: u64,

    /// Number of finished actions to keep as history.
    #[serde(default = "ActionsConfig::default_prune_keep")]
    pub prune_keep: u32,

    /// Number of finished actions to prune from the history in one cycle.
    #[serde(default = "ActionsConfig::default_prune_limit")]
    pub prune_limit: u32,
}

impl Default for ActionsConfig {
    fn default() -> Self {
        ActionsConfig {
            enabled: None,
            execute_interval: Self::default_execute_interval(),
            prune_interval: Self::default_prune_interval(),
            prune_keep: Self::default_prune_keep(),
            prune_limit: Self::default_prune_limit(),
        }
    }
}

impl ActionsConfig {
    fn default_execute_interval() -> u64 {
        1
    }

    fn default_prune_interval() -> u64 {
        3600
    }

    fn default_prune_keep() -> u32 {
        100
    }

    fn default_prune_limit() -> u32 {
        500
    }
}
