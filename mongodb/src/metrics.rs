use prometheus::CounterVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::Logger;


lazy_static! {
    pub static ref MONGO_COMMAND_COUNTS: CounterVec = CounterVec::new(
        Opts::new(
            "replicante_mongodb_commands",
            "Counts the commands executed against the MongoDB node"
        ),
        &["command"]
    ).expect("Unable to configure commands counter");
}


/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(MONGO_COMMAND_COUNTS.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MONGO_COMMAND_COUNTS"; "error" => error);
    }
}
