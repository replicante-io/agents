use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;
use jmx::MBeanThreadedClientOptions;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::utils::FailSpan;

use replicante_agent::AgentContext;
use replicante_agent::Result;

use super::super::errors::to_agent;
use super::super::metrics::OPS_COUNT;
use super::super::metrics::OPS_DURATION;
use super::super::metrics::OP_ERRORS_COUNT;
use super::super::metrics::RECONNECT_COUNT;


const KAFKA_BROKER_ID_MBEAN_QUERY: &'static str = "kafka.server:type=app-info,id=*";
const KAFKA_BROKER_VERSION: &'static str = "kafka.server:type=app-info";
const KAFKA_LAG_PREFIX: &'static str =
    "kafka.server:type=FetcherLagMetrics,name=ConsumerLag,clientId=ReplicaFetcherThread-0-";


/// Kafka specifics that rely on JMX.
pub struct KafkaJmx {
    context: AgentContext,
    jmx: MBeanThreadedClient,
    reconnect: AtomicBool,
    reconnect_address: MBeanAddress,
    reconnect_options: MBeanThreadedClientOptions,
}

impl KafkaJmx {
    pub fn new(context: AgentContext, target: String) -> Result<KafkaJmx> {
        let address = MBeanAddress::address(target);
        let options = MBeanThreadedClientOptions::default()
            // Limit the number of pending JMX requests to avoid memory exhaustion.
            .requests_buffer_size(1042);
        let jmx = MBeanThreadedClient::connect_with_options(
            address.clone(), options.clone()
        ).map_err(to_agent)?;
        Ok(KafkaJmx {
            context,
            jmx,
            reconnect: AtomicBool::new(false),
            reconnect_address: address,
            reconnect_options: options,
        })
    }

    /// Fetch the ID of the broker.
    pub fn broker_name(&self, parent: &mut Span) -> Result<String> {
        let mut names = {
            let mut span = self.context.tracer.span("brokerName").auto_finish();
            span.child_of(parent.context().clone());
            span.tag("service", "jmx");
            self.reconnect_if_needed(&mut span).fail_span(&mut span)?;
            span.log(Log::new().log("span.kind", "client-send"));
            OPS_COUNT.with_label_values(&["jmx", "queryNames"]).inc();
            let timer = OPS_DURATION.with_label_values(&["jmx", "queryNames"]).start_timer();
            let names = self.jmx.query_names(KAFKA_BROKER_ID_MBEAN_QUERY, "")
                .fail_span(&mut span)
                .map_err(|error| {
                    OP_ERRORS_COUNT.with_label_values(&["jmx", "queryNames"]).inc();
                    to_agent(error)
                });
            timer.observe_duration();
            span.log(Log::new().log("span.kind", "client-receive"));
            let names = self.check_jmx_response(names)?;
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

    /// Fetch the version of the broker.
    pub fn broker_version(&self, parent: &mut Span) -> Result<String> {
        let mut span = self.context.tracer.span("brokerVersion").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "jmx");
        self.reconnect_if_needed(&mut span).fail_span(&mut span)?;
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
        let timer = OPS_DURATION.with_label_values(&["jmx", "getAttribute"]).start_timer();
        let version = self.jmx.get_attribute(KAFKA_BROKER_VERSION, "version")
            .fail_span(&mut span)
            .map_err(|error| {
                OP_ERRORS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
                to_agent(error)
            });
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        let version = self.check_jmx_response(version)?;
        Ok(version)
    }

    /// Fetch replica lag information.
    pub fn replica_lag(
        &self, topic: &str, partition: i32, leader: i32, parent: &mut Span
    ) -> Result<i64> {
        let mut span = self.context.tracer.span("replicaLag").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "jmx");
        let key = format!(
            "{}{},topic={},partition={}", KAFKA_LAG_PREFIX, leader, topic, partition
        );
        self.reconnect_if_needed(&mut span).fail_span(&mut span)?;
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
        let timer = OPS_DURATION.with_label_values(&["jmx", "getAttribute"]).start_timer();
        let lag = self.jmx.get_attribute(key, "Value")
            .fail_span(&mut span)
            .map_err(|error| {
                OP_ERRORS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
                to_agent(error)
            });
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        let lag = self.check_jmx_response(lag)?;
        Ok(lag)
    }
}

impl KafkaJmx {
    /// Check if JMX responded with an error.
    ///
    /// If so, flag the need to reconnect so it can be done prior to the next request.
    fn check_jmx_response<T>(&self, result: Result<T>) -> Result<T> {
        if result.is_err() {
            self.reconnect.store(true, Ordering::Relaxed);
        }
        result
    }

    /// If there was an error in a past connection to the JMX server reconnect.
    fn reconnect_if_needed(&self, span: &mut Span) -> Result<()> {
        if self.reconnect.load(Ordering::Relaxed) {
            debug!(self.context.logger, "Reconnecting to JMX server");
            span.log(Log::new().log("action", "jmx.connect"));
            RECONNECT_COUNT.with_label_values(&["jmx"]).inc();
            self.jmx.reconnect_with_options(
                self.reconnect_address.clone(), self.reconnect_options.clone()
            ).map_err(to_agent)?;
            self.reconnect.store(false, Ordering::Relaxed);
            info!(self.context.logger, "Reconnected to JMX server");
        }
        Ok(())
    }
}
