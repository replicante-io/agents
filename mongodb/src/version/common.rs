use lazy_static::lazy_static;

use replicante_models_agent::info::AgentVersion;

lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_TAINT"),
    );
}
