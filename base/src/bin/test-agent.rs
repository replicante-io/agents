extern crate replicante_agent;

use replicante_agent::Agent;
use replicante_agent::AgentResult;
use replicante_agent::AgentRunner;

use replicante_agent::config::AgentConfig;

use replicante_agent::models::AgentVersion;
use replicante_agent::models::DatastoreVersion;
use replicante_agent::models::Shard;
use replicante_agent::models::ShardRole;


pub struct TestAgent {}

impl TestAgent {
    pub fn new() -> TestAgent {
        TestAgent {}
    }
}

impl Agent for TestAgent {
    fn datastore_version(&self) -> AgentResult<DatastoreVersion> {
        Ok(DatastoreVersion::new("Test DB", "1.2.3"))
    }

    fn shards(&self) -> AgentResult<Vec<Shard>> {
        Ok(vec![
            Shard::new("test-shard", ShardRole::Primary, 1, 2)
        ])
    }
}


fn main() {
    let runner = AgentRunner::new(
        Box::new(TestAgent::new()),
        AgentConfig::default(),
        AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
            env!("GIT_BUILD_TAINT")
        )
    );
    runner.run();
}
