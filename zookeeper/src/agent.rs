use opentracingrust::Span;

use replicante_agent::Agent;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;


/// Zookeeper 3.3+ agent.
pub struct ZookeeperAgent {
    // TODO
}

impl ZookeeperAgent {
    pub fn new() -> ZookeeperAgent {
        ZookeeperAgent {}
    }
}

impl Agent for ZookeeperAgent {
    fn agent_info(&self, _span: &mut Span) -> Result<AgentInfo> {
        Err("TODO".into())
    }

    fn datastore_info(&self, _span: &mut Span) -> Result<DatastoreInfo> {
        Err("TODO".into())
    }

    fn shards(&self, _span: &mut Span) -> Result<Shards> {
        Err("TODO".into())
    }
}
