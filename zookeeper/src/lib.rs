extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate slog;
extern crate zk_4lw;

extern crate replicante_agent;
extern crate replicante_agent_models;
extern crate replicante_util_tracing;

use replicante_agent::Result;

mod agent;
mod config;
mod error;
mod metrics;
mod zk4lw;

use agent::ZookeeperAgent;
use config::Config;

lazy_static! {
    static ref RELEASE: String = format!("repliagent-officials@{}", env!("GIT_BUILD_HASH"));
    pub static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT")
    );
}

const DEFAULT_CONFIG_FILE: &str = "agent-zookeeper.yaml";

/// Configure and start the agent.
pub fn run() -> Result<bool> {
    // Command line parsing.
    let cli_args = ::replicante_agent::process::clap(
        "Zookeeper Replicante Agent",
        VERSION.as_ref(),
        env!("CARGO_PKG_DESCRIPTION"),
        DEFAULT_CONFIG_FILE,
    )
    .get_matches();

    // Load configuration.
    Config::override_defaults();
    let config_location = cli_args.value_of("config").unwrap();
    let config = Config::from_file(config_location)?;
    let config = config.transform();

    // Run the agent using the provided default helper.
    let agent_conf = config.agent.clone();
    let release = RELEASE.as_str();
    ::replicante_agent::process::run(agent_conf, "repliagent-zookeeper", release, |context, _| {
        metrics::register_metrics(context);
        let agent = ZookeeperAgent::new(config, context.clone());
        Ok(agent)
    })
}
