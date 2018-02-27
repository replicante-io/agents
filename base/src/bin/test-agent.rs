extern crate opentracingrust;
extern crate replicante_agent;

use std::time::Duration;

use opentracingrust::Span;
use opentracingrust::Tracer;
use opentracingrust::tracers::NoopTracer;
use opentracingrust::utils::ReporterThread;

use replicante_agent::Agent;
use replicante_agent::AgentResult;
use replicante_agent::AgentRunner;

use replicante_agent::config::AgentConfig;

use replicante_agent::models::AgentVersion;
use replicante_agent::models::DatastoreVersion;
use replicante_agent::models::Shard;
use replicante_agent::models::ShardRole;


pub struct TestAgent {
    tracer: Tracer,
}

impl TestAgent {
    pub fn new(tracer: Tracer) -> TestAgent {
        TestAgent { tracer }
    }
}

impl Agent for TestAgent {
    fn agent_version(&self, _: &mut Span) -> AgentResult<AgentVersion> {
        Ok(AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
        ))
    }

    fn datastore_version(&self, _: &mut Span) -> AgentResult<DatastoreVersion> {
        Ok(DatastoreVersion::new("Test DB", "1.2.3"))
    }

    fn tracer(&self) -> &Tracer {
        &self.tracer
    }

    fn shards(&self, _: &mut Span) -> AgentResult<Vec<Shard>> {
        Ok(vec![
            Shard::new("test-shard", ShardRole::Primary, 1, 2)
        ])
    }
}


fn main() {
    // Setup and run the tracer.
    let (tracer, receiver) = NoopTracer::new();
    let mut reporter = ReporterThread::new(receiver, |span| {
        NoopTracer::report(span);
    });
    reporter.stop_delay(Duration::from_secs(2));

    // Setup and run the agent.
    let agent = TestAgent::new(tracer);
    let runner = AgentRunner::new(Box::new(agent), AgentConfig::default());
    runner.run();
}
