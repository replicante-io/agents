extern crate opentracingrust;
extern crate replicante_agent;
extern crate replicante_agent_mongodb;

use std::time::Duration;

use replicante_agent::AgentRunner;
use replicante_agent::config::configure_tracer;

use replicante_agent_mongodb::MongoDBAgent;
use replicante_agent_mongodb::settings::MongoDBAgentSettings;


fn main() {
    // Load settings for the agent.
    let mut settings = MongoDBAgentSettings::default();
    settings.load(vec![
        "agent-mongodb.yaml",
        "agent-mongodb-rs.yaml"
    ]).expect("Unable to load user settings");

    // Setup and run the tracer.
    let (tracer, mut reporter) = configure_tracer(settings.agent().tracer)
        .expect("Failed to initialise distributed tracer");
    reporter.stop_delay(Duration::from_secs(2));

    // Setup and run the agent.
    let agent = MongoDBAgent::new(settings.mongo(), tracer)
        .expect("Failed to initialise agent");
    let runner = AgentRunner::new(Box::new(agent), settings.agent());
    runner.run();
}
