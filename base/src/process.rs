use std::borrow::Cow;
use std::env;
use std::process::exit;

use clap::App;
use clap::Arg;
use failure::ResultExt;
use prometheus::process_collector::ProcessCollector;
use sentry::integrations::failure::capture_fail;
use sentry::internals::ClientInitGuard;
use sentry::internals::IntoDsn;
use slog::debug;
use slog::info;
use slog::warn;
use slog::Logger;
use slog_scope::GlobalLoggerGuard;

use replicante_util_failure::format_fail;
use replicante_util_tracing::tracer;
use replicante_util_upkeep::Upkeep;

use super::api;
use super::config::Agent as Config;
use super::config::SentryConfig;
use super::Agent;
use super::AgentContext;
use super::ErrorKind;
use super::Result;

/// Configure a command line parser.
///
/// The parser is configure with all the arguments every agent is required to implement.
/// Additional arguments can be added by each agent if needed.
pub fn clap<'a, 'b, S1, S2, S3>(
    name: S1,
    version: S2,
    description: S3,
    default_config_location: &'a str,
) -> App<'a, 'b>
where
    S1: Into<String>,
    S2: Into<&'b str>,
    S3: Into<&'b str>,
{
    App::new(name).version(version).about(description).arg(
        Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .default_value(default_config_location)
            .help("Specifies the configuration file to use")
            .takes_value(true),
    )
}

/// Main logic for the `run` function.
///
/// This function is implemented separately to allow `run` to apply error handling
/// once to all possible error return branches.
fn initialise_and_run<A, F>(
    config: Config,
    logger: Logger,
    service: &'static str,
    initialise: F,
) -> Result<bool>
where
    A: Agent + 'static,
    F: FnOnce(&AgentContext, &mut Upkeep) -> Result<A>,
{
    let mut upkeep = Upkeep::new();
    upkeep.set_logger(logger.clone());
    upkeep
        .register_signal()
        .with_context(|_| ErrorKind::Initialisation("signal handler registration failed".into()))?;

    let tracer_opts = replicante_util_tracing::Opts::new(service, logger.clone(), &mut upkeep);
    let tracer = tracer(config.tracing.clone(), tracer_opts)
        .with_context(|_| ErrorKind::Initialisation("tracer configuration failed".into()))?;

    let context = AgentContext::new(config, logger.clone(), tracer);
    register_process_metrics(&context);
    super::register_metrics(&context);
    let agent = initialise(&context, &mut upkeep)?;
    api::spawn_server(agent, context, &mut upkeep)?;
    let clean_exit = upkeep.keepalive();
    if clean_exit {
        info!(logger, "Agent stopped gracefully");
    } else {
        warn!(logger, "Exiting due to error in a worker thread");
    }

    Ok(clean_exit)
}

/// Configure and instantiate the logger.
pub fn logger(config: &Config) -> (Logger, GlobalLoggerGuard) {
    let logger_opts = ::replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
    let logger = ::replicante_logging::configure(config.logging.clone(), &logger_opts);
    let scope_guard = slog_scope::set_global_logger(logger.clone());
    slog_stdlog::init().expect("Failed to initialise log -> slog integration");
    (logger, scope_guard)
}

/// Easy entrypoint function to setup the environment and handle errors.
pub fn main<F>(run: F)
where
    F: FnOnce() -> Result<bool>,
{
    // Enable backtraces if the user did not set them.
    let have_rust = env::var("RUST_BACKTRACE").is_ok();
    let have_failure = env::var("RUST_FAILURE_BACKTRACE").is_ok();
    if !have_rust && !have_failure {
        env::set_var("RUST_FAILURE_BACKTRACE", "1");
    }

    // Can now run replicante.
    let result = run();
    match result {
        Err(error) => {
            let message = format_fail(&error);
            println!("{}", message);
            exit(1);
        }
        Ok(clean) if !clean => exit(1),
        _ => (),
    };
}

/// Register default process metrics.
pub fn register_process_metrics(context: &AgentContext) {
    let logger = &context.logger;
    let process = ProcessCollector::for_self();
    let registry = &context.metrics;
    if let Err(error) = registry.register(Box::new(process)) {
        debug!(logger, "Failed to register process metrics"; "error" => ?error);
    }
}

/// Run the agent process.
///
/// This function initialises all needed components and pipes them together.
///
/// Once done, the process blocks until shutdown is initiated.
/// See `replicante_util_upkeep::Upkeep` for details on blocking and shutdown.
pub fn run<A, F, R>(
    config: Config,
    service: &'static str,
    release: R,
    initialise: F,
) -> Result<bool>
where
    A: Agent + 'static,
    F: FnOnce(&AgentContext, &mut Upkeep) -> Result<A>,
    R: Into<Cow<'static, str>>,
{
    let (logger, _scope_guard) = logger(&config);
    let _sentry = sentry(config.sentry.clone(), &logger, release.into())?;
    initialise_and_run(config, logger, service, initialise).map_err(|error| {
        capture_fail(&error);
        error
    })
}

/// Initialise sentry integration.
///
/// If sentry is configured, the panic handler is also registered.
pub fn sentry(
    config: Option<SentryConfig>,
    logger: &Logger,
    release: Cow<'static, str>,
) -> Result<ClientInitGuard> {
    let config = match config {
        None => {
            info!(logger, "Not using sentry: no configuration provided");
            return Ok(sentry::init(()));
        }
        Some(config) => config,
    };
    info!(logger, "Configuring sentry integration");
    let dsn = config
        .dsn
        .into_dsn()
        .with_context(|_| ErrorKind::Initialisation("invalid sentry configuration".into()))?;
    let client = sentry::init(sentry::ClientOptions {
        attach_stacktrace: true,
        dsn,
        in_app_include: vec!["replicante", "replicante_agent"],
        release: Some(release),
        ..Default::default()
    });
    if client.is_enabled() {
        sentry::integrations::panic::register_panic_handler();
    }
    Ok(client)
}
