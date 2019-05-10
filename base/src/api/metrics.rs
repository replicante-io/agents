use prometheus::CounterVec;
use prometheus::HistogramVec;

use replicante_util_iron::MetricsMiddleware;

use super::super::AgentContext;

lazy_static! {
    pub static ref MIDDLEWARE: (HistogramVec, CounterVec, CounterVec) =
        MetricsMiddleware::metrics("repliagent");
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
}
