use std::collections::HashMap;
use std::time::Duration;

use serde_json;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::utils::FailSpan;

use zookeeper::ZooKeeper;

use replicante_agent::AgentContext;
use replicante_agent::Error;
use replicante_agent::Result;

use super::super::errors::to_agent;


const TOPICS_PATH: &'static str = "/brokers/topics";


/// Kafka specifics that rely on Zookeeper.
pub struct KafkaZoo {
    context: AgentContext,
    keeper: ZooKeeper,
}

impl KafkaZoo {
    pub fn new(context: AgentContext, target: String, timeout: u64) -> Result<KafkaZoo> {
        let timeout = Duration::from_secs(timeout);
        let keeper = ZooKeeper::connect(&target, timeout, |_| {}).map_err(to_agent)?;
        Ok(KafkaZoo {
            context,
            keeper,
        })
    }

    /// Fetch partitions metadata for the topic that are on the given broker.
    pub fn partitions(
        &self, broker: i32, topic: &str, parent: &mut Span
    ) -> Result<Vec<PartitionMeta>> {
        let mut span = self.context.tracer.span("partitions").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "zookeeper");
        span.log(Log::new().log("span.kind", "client-send"));
        let path = format!("{}/{}", TOPICS_PATH, topic);
        let (meta, _) = self.keeper.get_data(&path, false)
            .fail_span(&mut span)
            .map_err(to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        let mut partitions = Vec::new();
        let meta: PartitionsMap = serde_json::from_slice(&meta).map_err(to_agent)?;
        for (partition, brokers) in meta.partitions {
            if !brokers.contains(&broker) {
                continue;
            }
            let leader = brokers.first().ok_or_else(
                || Error::from(format!("Partition {} has no brokers", partition))
            )?.clone();
            partitions.push(PartitionMeta {
                leader,
                partition: partition.parse().map_err(to_agent)?,
                replicas: brokers,
            });
        }
        Ok(partitions)
    }

    /// Fetch a list of topics in the cluster.
    pub fn topics(&self, parent: &mut Span) -> Result<Vec<String>> {
        let mut span = self.context.tracer.span("topics").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "zookeeper");
        span.log(Log::new().log("span.kind", "client-send"));
        let topics = self.keeper.get_children(TOPICS_PATH, false)
            .fail_span(&mut span)
            .map_err(to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(topics)
    }
}


#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct PartitionMeta {
    /// ID of the leader for the partition.
    pub leader: i32,

    /// ID of the partition.
    pub partition: i32,

    /// IDs of the brokers with an in-sync replica (including the leader).
    pub replicas: Vec<i32>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct PartitionsMap {
    /// Map of partitions to brokers.
    pub partitions: HashMap<String, Vec<i32>>,

    /// Metadata version? Expected to be 1.
    pub version: i32,
}
