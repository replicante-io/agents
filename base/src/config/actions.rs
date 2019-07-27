use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Actions configuration
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionsConfig {
    /// Enable/disable agent actions.
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl Default for ActionsConfig {
    fn default() -> Self {
        ActionsConfig { enabled: None }
    }
}
