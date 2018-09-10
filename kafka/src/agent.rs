use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;
use jmx::MBeanThreadedClientOptions;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::utils::FailSpan;

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
use super::errors::to_agent;


const KAFKA_BROKER_ID_MBEAN_QUERY: &'static str = "kafka.server:type=app-info,id=*";
const KAFKA_BROKER_VERSION: &'static str = "kafka.server:type=app-info";

lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// Kafka 1.0+ agent.
pub struct KafkaAgent {
    cluster: String,
    context: AgentContext,
    jmx: MBeanThreadedClient,
}

impl KafkaAgent {
    pub fn new(config: Config, context: AgentContext) -> Result<KafkaAgent> {
        let jmx = MBeanThreadedClient::connect_with_options(
            MBeanAddress::address(config.kafka.target.jmx),
            MBeanThreadedClientOptions::default()
                // Limit the number of pending JMX requests to avoid memory exhaustion.
                .requests_buffer_size(1042)
        ).map_err(to_agent)?;
        Ok(KafkaAgent {
            cluster: config.kafka.cluster,
            context,
            jmx,
        })
    }
}

impl KafkaAgent {
    fn name(&self, parent: &mut Span) -> Result<String> {
        let mut names = {
            let mut span = self.context.tracer.span("brokerName").auto_finish();
            span.child_of(parent.context().clone());
            span.log(Log::new().log("span.kind", "client-send"));
            let names = self.jmx.query_names(KAFKA_BROKER_ID_MBEAN_QUERY, "")
                .fail_span(&mut span)
                .map_err(to_agent)?;
            span.log(Log::new().log("span.kind", "client-receive"));
            names
        };
        let name: String = match names.len() {
            0 => return Err("No broker id reported in JMX".into()),
            1 => names.remove(0),
            _ => return Err("Too many broker ids reported in JMX".into()),
        };

        // Parse things like "kafka.server:type=app-info,id=2" in just the ID.
        let mut parts: Vec<&str> = name.splitn(2, ':').collect();
        let part: &str = match parts.len() {
            2 => parts.remove(1),
            _ => return Err(format!("Invalid mbean name ({}): no keys specified", name).into()),
        };
        for item in part.split(',') {
            let mut pair: Vec<&str> = item.splitn(2, '=').collect();
            let (key, value) = match pair.len() {
                2 => (pair.remove(0), pair.remove(0)),
                _ => return Err(
                    format!("Invalid mbean property ({}): no value found", item).into()
                ),
            };
            if key == "id" {
                return Ok(value.to_string());
            }
        }
        Err(format!("Unable to extract broker id (from {})", name).into())
    }

    fn version(&self, parent: &mut Span) -> Result<String> {
        let mut span = self.context.tracer.span("brokerVersion").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        let version = self.jmx.get_attribute(KAFKA_BROKER_VERSION, "version")
            .fail_span(&mut span)
            .map_err(to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(version)
    }
}

impl Agent for KafkaAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        let info = AgentInfo::new(AGENT_VERSION.clone());
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let name = self.name(span)?;
        let cluster = self.cluster.clone();
        let version = self.version(span)?;
        Ok(DatastoreInfo::new(cluster, "Kafka", name, version))
    }

    fn shards(&self, _span: &mut Span) -> Result<Shards> {
        Err("TODO".into())
    }
}
