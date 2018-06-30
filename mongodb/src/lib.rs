#[macro_use(bson, doc)]
extern crate bson;
extern crate clap;
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate mongodb;
extern crate opentracingrust;
extern crate prometheus;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[macro_use]
extern crate slog;

extern crate replicante_agent;
extern crate replicante_agent_models;
extern crate replicante_util_tracing;

use std::path::Path;
use std::time::Duration;

use clap::App;
use clap::Arg;

use slog::Discard;
use slog::Logger;

use replicante_agent::AgentRunner;
use replicante_agent::Result;
use replicante_agent::ResultExt;

use replicante_util_tracing::TracerExtra;
use replicante_util_tracing::tracer;

mod agent;
mod config;
mod errors;
mod rs_status;

use agent::MongoDBAgent;
use config::Config;


lazy_static! {
    /// Version string.
    pub static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
}


const DEFAULT_CONFIG_FILE: &'static str = "agent-mongodb.yaml";


/// Configure and start the agent.
pub fn run() -> Result<()> {
    // Command line parsing.
    let cli_args = App::new("MongoDB Replicante Agent")
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
    let default_and_missing =
        config_location == DEFAULT_CONFIG_FILE &&
        !Path::new(DEFAULT_CONFIG_FILE).exists();
    let config = if default_and_missing {
        Config::default()
    } else {
        Config::from_file(DEFAULT_CONFIG_FILE)
            .chain_err(|| "Unable to load user configuration")?
    };

    // TODO: setup logging properly.
    let logger = Logger::root(Discard, o!());

    // Setup and run the tracer.
    let (tracer, mut extra) = tracer(config.agent.tracer.clone(), logger)
        .chain_err(|| "Unable to configure distributed tracer")?;
    match extra {
        TracerExtra::ReporterThread(ref mut reporter) => {
            reporter.stop_delay(Duration::from_secs(2));
        },
        _ => ()
    };

    // Setup and run the agent.
    let agent_config = config.agent.clone();
    let agent = MongoDBAgent::new(config, tracer)
        .chain_err(|| "Failed to initialise MongoDB agent")?;
    let runner = AgentRunner::new(agent, agent_config);
    runner.run();
    Ok(())
}
