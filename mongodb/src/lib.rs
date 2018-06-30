#[macro_use(bson, doc)]
extern crate bson;
extern crate error_chain;

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


const DEFAULT_CONFIG_FILE: &'static str = "agent-mongodb.yaml";


/// Configure and start the agent.
pub fn run() -> Result<()> {
    // Load configuration (default file is allowed to be missing).
    Config::override_defaults();
    let config = if Path::new(DEFAULT_CONFIG_FILE).exists() {
        Config::from_file(DEFAULT_CONFIG_FILE)
            .chain_err(|| "Unable to load user configuration")?
    } else {
        Config::default()
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
