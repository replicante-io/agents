use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Service supervisor configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "supervisor", content = "options")]
pub enum ServiceConfig {
    /// Control a service through execution of custom commands.
    #[serde(rename = "commands")]
    Commands(CommandsSupervisor),

    /// Control a service through `systemctl`.
    #[serde(rename = "systemd")]
    Systemd(SystemdSupervisor),
}

/// Custom commands supervisor configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CommandsSupervisor {
    /// Command to return the main PID of the datastore service.
    pub pid: Vec<String>,

    /// Command to start the datastore service.
    pub start: Vec<String>,

    /// Command to stop the datastore service.
    pub stop: Vec<String>,
}

/// Systemd-specific configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SystemdSupervisor {
    /// Option name of the service to manage.
    pub service_name: String,
}
