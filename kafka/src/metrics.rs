use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    /// Counter for Kafka/JMX/Zookeeper operation errors.
    pub static ref OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_agent_kafka_operation_errors",
            "Number of Kafka/JMX/Zookeeper operations failed"
        ),
        &["service", "operation"]
    ).expect("Failed to create OP_ERRORS_COUNT counter");

    /// Counter for Kafka/JMX/Zookeeper operations.
    pub static ref OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_agent_kafka_operations",
            "Number of Kafka/JMX/Zookeeper operations issued"
        ),
        &["service", "operation"]
    ).expect("Failed to create OPS_COUNT counter");

    /// Observe duration of Kafka/JMX/Zookeeper operations.
    pub static ref OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicante_agent_kafka_operations_duration",
            "Duration (in seconds) of Kafka/JMX/Zookeeper operations"
        ),
        &["service", "operation"]
    ).expect("Failed to create OPS_DURATION histogram");

    /// Counter for Kafka/JMX/Zookeeper reconnect operations.
    pub static ref RECONNECT_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_agent_kafka_reconnect",
            "Number of Kafka/JMX/Zookeeper reconnect operations"
        ),
        &["service"]
    ).expect("Failed to create RECONNECT_COUNT counter");
}


/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(OPS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register OPS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(OP_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register OP_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(OPS_DURATION.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register OPS_DURATION"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(RECONNECT_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register RECONNECT_COUNT"; "error" => error);
    }
}
