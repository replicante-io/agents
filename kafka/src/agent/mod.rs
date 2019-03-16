use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use failure::ResultExt;
use failure::SyncFailure;
use kafka::client::FetchOffset;
use kafka::client::KafkaClient;
use opentracingrust::Span;

use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::CommitOffset;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shard;
use replicante_agent_models::ShardRole;
use replicante_agent_models::Shards;

use super::Config;
use super::error::ErrorKind;
use super::metrics::OPS_COUNT;
use super::metrics::OPS_DURATION;
use super::metrics::OP_ERRORS_COUNT;


mod jmx;
mod zk;

use self::jmx::KafkaJmx;
use self::zk::KafkaZoo;


lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// Kafka 1.0+ agent.
pub struct KafkaAgent {
    jmx: KafkaJmx,
    kafka: Mutex<KafkaClient>,
    zoo: KafkaZoo,
}

impl KafkaAgent {
    pub fn with_config(config: Config, context: AgentContext) -> Result<KafkaAgent> {
        let jmx = KafkaJmx::with_context(context.clone(), config.kafka.target.jmx)?;
        let kafka_timeout = Duration::from_secs(config.kafka.target.broker.timeout);
        let mut kafka = KafkaClient::new(vec![config.kafka.target.broker.uri]);
        kafka.set_client_id("replicante-kafka-agent".into());
        kafka.set_fetch_max_wait_time(kafka_timeout)
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::ConfigOption("kafka.target.broker.timeout"))?;
        kafka.set_connection_idle_timeout(kafka_timeout);
        let zoo = KafkaZoo::connect(
            context,
            config.kafka.target.zookeeper.uri, config.kafka.target.zookeeper.timeout
        )?;
        Ok(KafkaAgent {
            jmx,
            kafka: Mutex::new(kafka),
            zoo,
        })
    }
}

impl KafkaAgent {
    /// Generate shard information for partitions of the given topic that are on this broker.
    fn push_shard(
        &self, shards: &mut Vec<Shard>, broker_id: i32, topic: &str, span: &mut Span
    ) -> Result<()> {
        let offsets = self.topic_offsets(topic, span)?;
        let partitions = self.zoo.partitions(broker_id, topic, span)?;
        for meta in partitions {
            let primary = meta.leader == broker_id;
            let role = if primary {
                ShardRole::Primary
            } else {
                ShardRole::Secondary
            };
            let id = format!("{}/{}", topic, meta.partition);
            let commit = if primary {
                offsets.get(&meta.partition).map(|offset| CommitOffset::unit(*offset, "offset"))
            } else {
                None
            };
            let lag = if primary {
                None
            } else {
                let lag = self.jmx.replica_lag(topic, meta.partition, meta.leader, span)?;
                Some(CommitOffset::unit(lag, "messages"))
            };
            shards.push(Shard::new(id, role, commit, lag));
        }
        Ok(())
    }

    /// Return the latest partition offsets for all partitions in the topic.
    fn topic_offsets(&self, topic: &str, _span: &mut Span) -> Result<HashMap<i32, i64>> {
        let mut client = self.kafka.lock().expect("Kafka client lock was poisoned");
        OPS_COUNT.with_label_values(&["kafka", "loadMetadata"]).inc();
        let timer = OPS_DURATION.with_label_values(&["kafka", "loadMetadata"]).start_timer();
        client.load_metadata(&[topic]).map_err(|error| {
            OP_ERRORS_COUNT.with_label_values(&["kafka", "loadMetadata"]).inc();
            SyncFailure::new(error)
        }).with_context(|_| ErrorKind::StoreOpFailed("loadMetadata"))?;
        timer.observe_duration();
        let offsets = client.fetch_offsets(&[topic], FetchOffset::Latest)
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::StoreOpFailed("fetch_offsets"))?;
        let offsets = offsets.get(topic)
            .ok_or_else(|| ErrorKind::TopicNoOffsets(topic.to_string()))?;
        Ok(offsets.iter().map(|item| (item.partition, item.offset)).collect())
    }
}

impl Agent for KafkaAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        let info = AgentInfo::new(AGENT_VERSION.clone());
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let cluster = self.zoo.cluster_id(span)?;
        let name = self.jmx.broker_name(span)?;
        let version = self.jmx.broker_version(span)?;
        Ok(DatastoreInfo::new(cluster, "Kafka", name, version))
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        let name = self.jmx.broker_name(span)?;
        let broker_id: i32 = name.parse::<i32>()
            .with_context(|_| ErrorKind::BrokerIdFormat(name))?;
        let mut shards = Vec::new();
        for topic in self.zoo.topics(span)? {
            self.push_shard(&mut shards, broker_id, &topic, span)?;
        }
        Ok(Shards::new(shards))
    }
}
