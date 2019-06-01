use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use slog::debug;

use replicante_agent::AgentContext;

lazy_static! {
    pub static ref MONGODB_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_mongodb_operation_errors",
            "Number of MongoDB operations failed"
        ),
        &["operation"]
    )
    .expect("Failed to create MONGODB_OP_ERRORS_COUNT counter");
    pub static ref MONGODB_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_mongodb_operations",
            "Number of MongoDB operations issued"
        ),
        &["operation"]
    )
    .expect("Failed to create MONGODB_OPS_COUNT counter");
    pub static ref MONGODB_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "repliagent_mongodb_operations_duration",
            "Duration (in seconds) of MongoDB operations"
        ),
        &["operation"]
    )
    .expect("Failed to create MONGODB_OPS_DURATION histogram");
}

/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(context: &AgentContext) {
    let logger = &context.logger;
    let registry = &context.metrics;
    if let Err(error) = registry.register(Box::new(MONGODB_OPS_COUNT.clone())) {
        debug!(logger, "Failed to register MONGODB_OPS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MONGODB_OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register MONGODB_OP_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MONGODB_OPS_DURATION.clone())) {
        debug!(logger, "Failed to register MONGODB_OPS_DURATION"; "error" => ?error);
    }
}
