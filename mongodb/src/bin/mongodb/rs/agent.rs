extern crate opentracingrust;
extern crate replicante_agent;
extern crate replicante_agent_mongodb;

use std::time::Duration;

use opentracingrust::tracers::NoopTracer;
use opentracingrust::utils::ReporterThread;

use replicante_agent::AgentRunner;

use replicante_agent_mongodb::MongoDBAgent;
use replicante_agent_mongodb::settings::MongoDBAgentSettings;


fn main() {
    // Setup and run the tracer.
    let (tracer, receiver) = NoopTracer::new();
    let mut reporter = ReporterThread::new(receiver, |span| {
        NoopTracer::report(span);
    });
    reporter.stop_delay(Duration::from_secs(2));

    // Setup and run the agent.
    let mut settings = MongoDBAgentSettings::default();
    settings.load(vec![
        "agent-mongodb.yaml",
        "agent-mongodb-rs.yaml"
    ]).expect("Unable to load user settings");
    let agent = MongoDBAgent::new(settings.mongo(), tracer)
        .expect("Failed to initialise agent");
    let runner = AgentRunner::new(Box::new(agent), settings.agent());
    runner.run();
}
