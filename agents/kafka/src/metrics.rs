use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use slog::debug;

use replicante_agent::AgentContext;

lazy_static! {
    pub static ref OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_kafka_operation_errors",
            "Number of Kafka/JMX/Zookeeper operations failed"
        ),
        &["service", "operation"]
    )
    .expect("Failed to create OP_ERRORS_COUNT counter");
    pub static ref OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_kafka_operations",
            "Number of Kafka/JMX/Zookeeper operations issued"
        ),
        &["service", "operation"]
    )
    .expect("Failed to create OPS_COUNT counter");
    pub static ref OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "repliagent_kafka_operations_duration",
            "Duration (in seconds) of Kafka/JMX/Zookeeper operations"
        ),
        &["service", "operation"]
    )
    .expect("Failed to create OPS_DURATION histogram");
    pub static ref RECONNECT_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_kafka_reconnect",
            "Number of Kafka/JMX/Zookeeper reconnect operations"
        ),
        &["service"]
    )
    .expect("Failed to create RECONNECT_COUNT counter");
}

/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(context: &AgentContext) {
    let logger = &context.logger;
    let registry = &context.metrics;
    if let Err(error) = registry.register(Box::new(OPS_COUNT.clone())) {
        debug!(logger, "Failed to register OPS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register OP_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(OPS_DURATION.clone())) {
        debug!(logger, "Failed to register OPS_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(RECONNECT_COUNT.clone())) {
        debug!(logger, "Failed to register RECONNECT_COUNT"; "error" => ?error);
    }
}
