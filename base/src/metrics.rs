use lazy_static::lazy_static;
use prometheus::CounterVec;
use prometheus::Gauge;
use prometheus::HistogramVec;
use slog::debug;

use replicante_util_iron::MetricsMiddleware;

use super::AgentContext;

lazy_static! {
    pub static ref MIDDLEWARE: (HistogramVec, CounterVec, CounterVec) =
        MetricsMiddleware::metrics("repliagent");
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
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.0.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.0"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.1.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.1"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(MIDDLEWARE.2.clone())) {
        debug!(logger, "Failed to register MIDDLEWARE.2"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(UPDATE_AVAILABLE.clone())) {
        debug!(logger, "Failed to register UPDATE_AVAILABLE"; "error" => ?error);
    }
}
