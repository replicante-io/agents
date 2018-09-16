use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use serde_json;
use slog::Logger;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::utils::FailSpan;

use zookeeper::ZkState;
use zookeeper::ZooKeeper;

use replicante_agent::AgentContext;
use replicante_agent::Error;
use replicante_agent::Result;

use super::super::errors::to_agent;


const CLUSTER_ID_PATH: &'static str = "/cluster/id";
const TOPICS_PATH: &'static str = "/brokers/topics";


#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct ClusterId {
    /// Id of the kafka cluster.
    pub id: String,

    /// Metadata version? Expected to be 1.
    pub version: String,
}


/// Kafka specifics that rely on Zookeeper.
pub struct KafkaZoo {
    context: AgentContext,
    session: Mutex<ZookeeperSession>,
    target: String,
    timeout: Duration,
}

impl KafkaZoo {
    pub fn new(context: AgentContext, target: String, timeout: u64) -> Result<KafkaZoo> {
        let timeout = Duration::from_secs(timeout);
        let session = ZookeeperSession::new(&target, timeout.clone(), context.logger.clone())?;
        Ok(KafkaZoo {
            context,
            session: Mutex::new(session),
            target,
            timeout,
        })
    }

    /// Fetch the ID of the cluster.
    pub fn cluster_id(&self, parent: &mut Span) -> Result<String> {
        let mut span = self.context.tracer.span("clusterId").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "zookeeper");
        span.log(Log::new().log("span.kind", "client-send"));
        let keeper = self.keeper(&mut span).fail_span(&mut span)?;
        let (id, _) = keeper.get_data(CLUSTER_ID_PATH, false)
            .fail_span(&mut span)
            .map_err(to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        let id: ClusterId = serde_json::from_slice(&id).map_err(to_agent)?;
        Ok(id.id)
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
        let keeper = self.keeper(&mut span).fail_span(&mut span)?;
        let (meta, _) = keeper.get_data(&path, false)
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
        let keeper = self.keeper(&mut span).fail_span(&mut span)?;
        let topics = keeper.get_children(TOPICS_PATH, false)
            .fail_span(&mut span)
            .map_err(to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(topics)
    }
}

impl KafkaZoo {
    /// Grab a zookeeper session, re-creating it if needed.
    fn keeper(&self, span: &mut Span) -> Result<Arc<ZooKeeper>> {
        let mut session = self.session.lock().expect("Zookeeper session lock was poisoned");
        if !session.active() {
            debug!(self.context.logger, "Creating new zookeeper session");
            span.log(Log::new().log("action", "zookeeper.connect"));
            let new_session = ZookeeperSession::new(
                &self.target, self.timeout.clone(), self.context.logger.clone()
            )?;
            *session = new_session;
            info!(self.context.logger, "New zookeeper session ready");
        }
        Ok(session.client())
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


/// Container for a zookeeper session.
struct ZookeeperSession {
    active: Arc<AtomicBool>,
    client: Arc<ZooKeeper>,
}

impl ZookeeperSession {
    /// Create a new zookeeper session.
    pub fn new(connection: &str, timeout: Duration, logger: Logger) -> Result<ZookeeperSession> {
        let client = ZooKeeper::connect(connection, timeout, |_| {})
            .map_err(to_agent)?;
        let active = Arc::new(AtomicBool::new(true));
        let notify_close = Arc::clone(&active);
        client.add_listener(move |state| {
            let reset = match state {
                ZkState::AuthFailed => {
                    error!(logger, "Zookeeper authentication error");
                    false
                },
                ZkState::Closed => {
                    warn!(logger, "Zookeeper session closed");
                    true
                },
                ZkState::Connected => {
                    info!(logger, "Zookeeper connection successfull");
                    false
                },
                ZkState::ConnectedReadOnly => {
                    warn!(logger, "Zookeeper connection is read-only");
                    false
                },
                ZkState::Connecting => {
                    debug!(logger, "Zookeeper session connecting");
                    false
                },
                event => {
                    debug!(logger, "Ignoring deprecated zookeeper event"; "event" => ?event);
                    false
                },
            };
            if reset {
                notify_close.store(false, Ordering::Relaxed);
                debug!(logger, "Zookeeper session marked as not active");
            }
        });
        let client = Arc::new(client);
        Ok(ZookeeperSession {
            active,
            client,
        })
    }

    /// Checks if the session is active.
    ///
    /// A session is active if the connection to ZooKeper is intact.
    ///
    /// There may be some time while the connection is broken but the session is marked as
    /// active while the client tries to re-establish the connection.
    /// If this cannot be done, the session is marked as not active.
    pub fn active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Get a reference to the ZooKeeper client for this session.
    pub fn client(&self) -> Arc<ZooKeeper> {
        Arc::clone(&self.client)
    }
}
