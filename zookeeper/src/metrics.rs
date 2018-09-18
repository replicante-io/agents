use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;


lazy_static! {
    /// Counter for Zookeeper operation errors.
    pub static ref OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_agent_zookeeper_operation_errors",
            "Number of Zookeeper operations failed"
        ),
        &["operation"]
    ).expect("Failed to create OP_ERRORS_COUNT counter");

    /// Counter for Zookeeper operations.
    pub static ref OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_agent_zookeeper_operations",
            "Number of Zookeeper operations issued"
        ),
        &["operation"]
    ).expect("Failed to create OPS_COUNT counter");

    /// Observe duration of Zookeeper operations.
    pub static ref OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicante_agent_zookeeper_operations_duration",
            "Duration (in seconds) of Zookeeper operations"
        ),
        &["operation"]
    ).expect("Failed to create OPS_DURATION histogram");
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
}
