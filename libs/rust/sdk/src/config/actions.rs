use serde::Deserialize;
use serde::Serialize;

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

/// Parameters of a user-defined external action.
///
/// External actions call out to other programs or script to perform their tasks.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ExternalActionConfig {
    /// Command to execute to start the action.
    ///
    /// The first element in the list is the command to run.
    /// All following elements in the list are optional and are passed to the command as arguments.
    ///
    /// The start command MUST return quickly and execute the action asynchronously.
    /// This allows the agent to move on to other tasks.
    ///
    /// A record for the action invocation to check is passed as JSON to standard input.
    /// This information can be used to access things like the action ID, usable as a unique
    /// reference, or arguments passed to the agent when the action was scheduled.
    pub action: Vec<String>,

    /// Command to execute to check on the state of the action.
    ///
    /// The first element in the list is the command to run.
    /// All following elements in the list are optional and are passed to the command as arguments.
    ///
    /// The check command MUST implement the following protocol:
    ///
    ///   * The check command MUST return quickly.
    ///     Long running actions should periodically generate status information for the check
    ///     command to look up and make decisions from.
    ///   * The check command MUST exit successfully (exit code 0) if it could determine the
    ///     state of the process, even if the process has failed.
    ///   * The check MUST report the state of the action as JSON sent to its standard output.
    ///     The expected JSON object is described below.
    ///   * If the check command exists unsuccessfully (exit code not 0) it is assume the action
    ///     state can no longer be determined and it has failed.
    ///   * A record for the action invocation to check is passed as JSON to standard input.
    ///
    /// The JSON report printed by the check to standard output must match the following:
    /// ```json
    /// {
    ///   "status": "running" | "finished" | "failed",
    ///   "error": <error message, required it status == failed>
    /// }
    /// ```
    pub check: Vec<String>,

    /// Operator friendly description of what the action does.
    pub description: String,
}
