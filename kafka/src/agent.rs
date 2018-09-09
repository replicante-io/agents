use opentracingrust::Span;

use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;
//use replicante_agent::ResultExt;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
//use replicante_agent_models::CommitOffset;
use replicante_agent_models::DatastoreInfo;
//use replicante_agent_models::Shard;
//use replicante_agent_models::ShardRole;
use replicante_agent_models::Shards;

use super::Config;


lazy_static! {
    pub static ref _AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// Kafka 1.0+ agent.
pub struct KafkaAgent {
    // TODO
}

impl KafkaAgent {
    pub fn new(_config: Config, _context: AgentContext) -> KafkaAgent {
        KafkaAgent {
            // TODO
        }
    }
}

impl Agent for KafkaAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        Err("TODO".into())
    }

    fn datastore_info(&self, _span: &mut Span) -> Result<DatastoreInfo> {
        Err("TODO".into())
    }

    fn shards(&self, _span: &mut Span) -> Result<Shards> {
        Err("TODO".into())
    }
}
