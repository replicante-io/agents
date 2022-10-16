use lazy_static::lazy_static;

use replicante_agent::Result;
use replicante_agent::SemVersion;

mod agent;
mod config;
mod error;
mod metrics;

use agent::KafkaAgent;
use config::Config;

const DEFAULT_CONFIG_FILE: &str = "agent-kafka.yaml";
const UPDATE_META: &str =
    "https://github.com/replicante-io/metadata/raw/main/replicante/agent/kafka/latest.json";
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

lazy_static! {
    static ref CURRENT_VERSION: SemVersion = SemVersion::parse(env!("CARGO_PKG_VERSION")).unwrap();
    static ref RELEASE: String = format!("repliagent-officials@{}", env!("GIT_BUILD_HASH"));
}

/// Configure and start the agent.
pub fn run() -> Result<bool> {
    // Command line parsing.
    let cli_args = ::replicante_agent::process::clap(
        "Kafka Replicante Agent",
        VERSION,
        env!("CARGO_PKG_DESCRIPTION"),
        DEFAULT_CONFIG_FILE,
    )
    .get_matches();

    // Load configuration.
    Config::override_defaults();
    let config_location: &String = cli_args
        .get_one("config")
        .expect("CLI arguments to have a config value");
    let config = Config::from_file(config_location)?;
    let config = config.transform();

    // Run the agent using the provided default helper.
    let agent_conf = config.agent.clone();
    let release = RELEASE.as_str();
    replicante_agent::process::run(agent_conf, "repliagent-kafka", release, |context, _| {
        metrics::register_metrics(context);
        let agent = KafkaAgent::with_config(config, context.clone())?;
        replicante_agent::process::update_checker(CURRENT_VERSION.clone(), UPDATE_META, context)?;
        Ok(agent)
    })
}
