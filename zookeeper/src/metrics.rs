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
            "repliagent_zookeeper_operation_errors",
            "Number of Zookeeper operations failed"
        ),
        &["operation"]
    )
    .expect("Failed to create OP_ERRORS_COUNT counter");
    pub static ref OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_zookeeper_operations",
            "Number of Zookeeper operations issued"
        ),
        &["operation"]
    )
    .expect("Failed to create OPS_COUNT counter");
    pub static ref OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "repliagent_zookeeper_operations_duration",
            "Duration (in seconds) of Zookeeper operations"
        ),
        &["operation"]
    )
    .expect("Failed to create OPS_DURATION histogram");
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
}
