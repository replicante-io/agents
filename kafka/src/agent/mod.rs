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


mod jmx;

use self::jmx::KafkaJmx;


lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// Kafka 1.0+ agent.
pub struct KafkaAgent {
    cluster: String,
    jmx: KafkaJmx,
}

impl KafkaAgent {
    pub fn new(config: Config, context: AgentContext) -> Result<KafkaAgent> {
        let jmx = KafkaJmx::new(context, config.kafka.target.jmx)?;
        Ok(KafkaAgent {
            cluster: config.kafka.cluster,
            jmx,
        })
    }
}

impl Agent for KafkaAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        let info = AgentInfo::new(AGENT_VERSION.clone());
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let cluster = self.cluster.clone();
        let name = self.jmx.broker_name(span)?;
        let version = self.jmx.broker_version(span)?;
        Ok(DatastoreInfo::new(cluster, "Kafka", name, version))
    }

    fn shards(&self, _span: &mut Span) -> Result<Shards> {
        Err("TODO".into())
    }
}
