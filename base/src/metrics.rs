use lazy_static::lazy_static;
use prometheus::Gauge;
use slog::debug;

use replicante_util_actixweb::MetricsCollector;

use crate::AgentContext;

lazy_static! {
    pub static ref REQUESTS: MetricsCollector = MetricsCollector::new("repliagent");
    pub static ref UPDATE_AVAILABLE: Gauge = Gauge::new(
        "repliagent_updateable",
        "Set to 1 when an updateded version is available (checked at start only)",
    )
    .expect("Failed to create UPDATE_AVAILABLE gauge");
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(context: &AgentContext) {
    let logger = &context.logger;
    let registry = &context.metrics;
    REQUESTS.register(logger, registry);
    if let Err(error) = registry.register(Box::new(UPDATE_AVAILABLE.clone())) {
        debug!(logger, "Failed to register UPDATE_AVAILABLE"; "error" => ?error);
    }
}
