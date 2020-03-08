use lazy_static::lazy_static;
use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::Gauge;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use slog::debug;

use replicante_util_actixweb::MetricsCollector;

use crate::AgentContext;

lazy_static! {
    pub static ref ACTION_COUNT: CounterVec = CounterVec::new(
        Opts::new("repliagent_action_total", "Number of actions invoked"),
        &["action"],
    )
    .expect("Failed to create ACTION_COUNT histogram");
    pub static ref ACTION_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "repliagent_action_duration",
            "Duration (in seconds) of an action invokation"
        ),
        &["action"],
    )
    .expect("Failed to create ACTION_DURATION histogram");
    pub static ref ACTION_ERRORS: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_action_errors",
            "Number of actions that errored while being invoked",
        ),
        &["action"],
    )
    .expect("Failed to create ACTION_ERRORS histogram");
    pub static ref ACTION_PRUNE_DURATION: Histogram = Histogram::with_opts(HistogramOpts::new(
        "repliagent_action_prune_duration",
        "Duration (in seconds) of actions DB pruning"
    ))
    .expect("Failed to create ACTION_DURATION histogram");
    pub static ref REQUESTS: MetricsCollector = MetricsCollector::new("repliagent");
    pub static ref SQLITE_CONNECTION_ERRORS: Counter = Counter::new(
        "repliagent_sqlite_connection_errors",
        "Number of SQLite connection errors",
    )
    .expect("Failed to create UPDATE_AVAILABLE gauge");
    pub static ref SQLITE_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_sqlite_operation_errors",
            "Number of SQLite operations failed",
        ),
        &["operation"],
    )
    .expect("Failed to create SQLITE_OP_ERRORS_COUNT counter");
    pub static ref SQLITE_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new(
            "repliagent_sqlite_operations",
            "Number of SQLite operations issued",
        ),
        &["operation"],
    )
    .expect("Failed to create SQLITE_OPS_COUNT counter");
    pub static ref SQLITE_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "repliagent_sqlite_operations_duration",
            "Duration (in seconds) of SQLite operations"
        ),
        &["operation"],
    )
    .expect("Failed to create SQLITE_OPS_DURATION histogram");
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
    if let Err(error) = registry.register(Box::new(ACTION_COUNT.clone())) {
        debug!(logger, "Failed to register ACTION_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACTION_DURATION.clone())) {
        debug!(logger, "Failed to register ACTION_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(ACTION_ERRORS.clone())) {
        debug!(logger, "Failed to register ACTION_ERRORS"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SQLITE_OP_ERRORS_COUNT.clone())) {
        debug!(logger, "Failed to register SQLITE_OP_ERRORS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SQLITE_OPS_COUNT.clone())) {
        debug!(logger, "Failed to register SQLITE_OPS_COUNT"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(SQLITE_OPS_DURATION.clone())) {
        debug!(logger, "Failed to register SQLITE_OPS_DURATION"; "error" => ?error);
    }
    if let Err(error) = registry.register(Box::new(UPDATE_AVAILABLE.clone())) {
        debug!(logger, "Failed to register UPDATE_AVAILABLE"; "error" => ?error);
    }
}
