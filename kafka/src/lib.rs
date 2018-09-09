extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate slog;

extern crate replicante_agent;
extern crate replicante_agent_models;
extern crate replicante_util_tracing;

use std::time::Duration;

use clap::App;
use clap::Arg;

use replicante_agent::AgentContext;
use replicante_agent::AgentRunner;
use replicante_agent::Result;
use replicante_agent::ResultExt;

use replicante_util_tracing::TracerExtra;
use replicante_util_tracing::tracer;

mod agent;
mod config;
//mod errors;
mod metrics;

use agent::KafkaAgent;
use config::Config;


lazy_static! {
    /// Version string.
    pub static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
}


const DEFAULT_CONFIG_FILE: &'static str = "agent-kafka.yaml";


/// Configure and start the agent.
pub fn run() -> Result<()> {
    // Command line parsing.
    let cli_args = App::new("Kafka Replicante Agent")
        .version(VERSION.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .default_value(DEFAULT_CONFIG_FILE)
             .help("Specifies the configuration file to use")
             .takes_value(true)
        )
        .get_matches();

    // Load configuration (default file is allowed to be missing).
    Config::override_defaults();
    let config_location = cli_args.value_of("config").unwrap();
    let config = Config::from_file(config_location)
        .chain_err(|| "Unable to load user configuration")?;

    // Configure the logger (from the agent context).
    let agent_config = config.agent.clone();
    let logger = AgentContext::logger(&agent_config);

    // Setup and run the tracer.
    let (tracer, mut extra) = tracer(config.agent.tracing.clone(), logger.clone())
        .chain_err(|| "Unable to configure distributed tracer")?;
    match extra {
        TracerExtra::ReporterThread(ref mut reporter) => {
            reporter.stop_delay(Duration::from_secs(2));
        },
        _ => ()
    };

    // Setup the agent context.
    let agent_context = AgentContext::new(agent_config, logger, tracer);
    AgentRunner::register_metrics(&agent_context.logger, &agent_context.metrics);
    metrics::register_metrics(&agent_context.logger, &agent_context.metrics);

    // Setup and run the agent.
    let agent = KafkaAgent::new(config, agent_context.clone());
    let runner = AgentRunner::new(agent, agent_context);
    runner.run();

    // Cleanup tracer and exit.
    drop(extra);
    Ok(())
}
