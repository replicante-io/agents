use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use failure::ResultExt;
use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;
use jmx::MBeanThreadedClientOptions;

use opentracingrust::Log;
use opentracingrust::Span;

use replicante_agent::fail_span;
use replicante_agent::AgentContext;
use replicante_agent::Error;
use replicante_agent::Result;

use super::super::error::ErrorKind;
use super::super::metrics::OPS_COUNT;
use super::super::metrics::OPS_DURATION;
use super::super::metrics::OP_ERRORS_COUNT;
use super::super::metrics::RECONNECT_COUNT;

const KAFKA_BROKER_ID_MBEAN_QUERY: &str = "kafka.server:type=app-info,id=*";
const KAFKA_BROKER_VERSION: &str = "kafka.server:type=app-info";
const KAFKA_LAG_PREFIX: &str =
    "kafka.server:type=FetcherLagMetrics,name=ConsumerLag,clientId=ReplicaFetcherThread-0-";

// Limit the number of pending JMX requests to avoid memory exhaustion.
const JMX_REQUESTS_QUEUE: usize = 1024;

/// Kafka specifics that rely on JMX.
pub struct KafkaJmx {
    context: AgentContext,
    jmx: MBeanThreadedClient,
    reconnect: AtomicBool,
    reconnect_address: MBeanAddress,
}

impl KafkaJmx {
    pub fn with_context(context: AgentContext, target: String) -> Result<KafkaJmx> {
        let address = MBeanAddress::address(target);
        let options = MBeanThreadedClientOptions::default()
            .requests_buffer_size(JMX_REQUESTS_QUEUE)
            // Skip connecting the first time around.
            .skip_connect(true);
        let jmx = MBeanThreadedClient::connect_with_options(address.clone(), options)
            .with_context(|_| {
                ErrorKind::JmxConnection(match address.clone() {
                    MBeanAddress::Address(address) => address,
                    MBeanAddress::ServiceUrl(address) => address,
                })
            })?;
        Ok(KafkaJmx {
            context,
            jmx,
            reconnect: AtomicBool::new(true),
            reconnect_address: address,
        })
    }

    /// Fetch the ID of the broker.
    pub fn broker_name(&self, parent: &mut Span) -> Result<String> {
        let mut names = {
            let mut span = self.context.tracer.span("brokerName").auto_finish();
            span.child_of(parent.context().clone());
            span.tag("service", "jmx");
            self.reconnect_if_needed(&mut span)
                .map_err(|error| fail_span(error, &mut span))?;
            span.log(Log::new().log("span.kind", "client-send"));
            OPS_COUNT.with_label_values(&["jmx", "queryNames"]).inc();
            let timer = OPS_DURATION
                .with_label_values(&["jmx", "queryNames"])
                .start_timer();
            let names = self
                .jmx
                .query_names(KAFKA_BROKER_ID_MBEAN_QUERY, "")
                .map_err(|error| {
                    OP_ERRORS_COUNT
                        .with_label_values(&["jmx", "queryNames"])
                        .inc();
                    fail_span(error, &mut span)
                })
                .with_context(|_| ErrorKind::StoreOpFailed("<jmx>.broker_name"))
                .map_err(Error::from);
            timer.observe_duration();
            span.log(Log::new().log("span.kind", "client-receive"));
            self.check_jmx_response(names)?
        };
        let name: String = match names.len() {
            0 => return Err(ErrorKind::BrokerNoId.into()),
            1 => names.remove(0),
            _ => return Err(ErrorKind::BrokerTooManyIds.into()),
        };

        // Parse things like "kafka.server:type=app-info,id=2" in just the ID.
        let mut parts: Vec<&str> = name.splitn(2, ':').collect();
        let part: &str = match parts.len() {
            2 => parts.remove(1),
            _ => return Err(ErrorKind::BrokerIdFormat(name.clone()).into()),
        };
        for item in part.split(',') {
            let mut pair: Vec<&str> = item.splitn(2, '=').collect();
            let (key, value) = match pair.len() {
                2 => (pair.remove(0), pair.remove(0)),
                _ => return Err(ErrorKind::BrokerIdFormat(item.to_string()).into()),
            };
            if key == "id" {
                return Ok(value.to_string());
            }
        }
        Err(ErrorKind::BrokerIdFormat(name.clone()).into())
    }

    /// Fetch the version of the broker.
    pub fn broker_version(&self, parent: &mut Span) -> Result<String> {
        let mut span = self.context.tracer.span("brokerVersion").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "jmx");
        self.reconnect_if_needed(&mut span)
            .map_err(|error| fail_span(error, &mut span))?;
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
        let timer = OPS_DURATION
            .with_label_values(&["jmx", "getAttribute"])
            .start_timer();
        let version = self
            .jmx
            .get_attribute(KAFKA_BROKER_VERSION, "version")
            .map_err(|error| {
                OP_ERRORS_COUNT
                    .with_label_values(&["jmx", "getAttribute"])
                    .inc();
                fail_span(error, &mut span)
            })
            .with_context(|_| ErrorKind::StoreOpFailed("<jmx>.broker_version"))
            .map_err(Error::from);
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        let version = self.check_jmx_response(version)?;
        Ok(version)
    }

    /// Fetch replica lag information.
    pub fn replica_lag(
        &self,
        topic: &str,
        partition: i32,
        leader: i32,
        parent: &mut Span,
    ) -> Result<i64> {
        let mut span = self.context.tracer.span("replicaLag").auto_finish();
        span.child_of(parent.context().clone());
        span.tag("service", "jmx");
        let key = format!(
            "{}{},topic={},partition={}",
            KAFKA_LAG_PREFIX, leader, topic, partition
        );
        self.reconnect_if_needed(&mut span)
            .map_err(|error| fail_span(error, &mut span))?;
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["jmx", "getAttribute"]).inc();
        let timer = OPS_DURATION
            .with_label_values(&["jmx", "getAttribute"])
            .start_timer();
        let lag = self
            .jmx
            .get_attribute(key, "Value")
            .map_err(|error| {
                OP_ERRORS_COUNT
                    .with_label_values(&["jmx", "getAttribute"])
                    .inc();
                fail_span(error, &mut span)
            })
            .with_context(|_| ErrorKind::StoreOpFailed("<jmx>.partitionLag"))
            .map_err(Error::from);
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
            let options = self.reconnect_options();
            self.jmx
                .reconnect_with_options(self.reconnect_address.clone(), options)
                .with_context(|_| {
                    ErrorKind::JmxConnection(match self.reconnect_address.clone() {
                        MBeanAddress::Address(address) => address,
                        MBeanAddress::ServiceUrl(address) => address,
                    })
                })?;
            self.reconnect.store(false, Ordering::Relaxed);
            info!(self.context.logger, "Reconnected to JMX server");
        }
        Ok(())
    }

    /// Generate connection options for reconnecting to the JMX server.
    fn reconnect_options(&self) -> MBeanThreadedClientOptions {
        MBeanThreadedClientOptions::default().requests_buffer_size(JMX_REQUESTS_QUEUE)
    }
}
