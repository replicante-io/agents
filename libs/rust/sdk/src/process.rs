use std::borrow::Cow;
use std::collections::BTreeMap;
use std::env;
use std::process::exit;

use clap::Arg;
use clap::Command;
use failure::ResultExt;
use humthreads::Builder;
use prometheus::process_collector::ProcessCollector;
use semver::Version;
use sentry::ClientInitGuard;
use sentry::IntoDsn;
use serde::Deserialize;
use slog::debug;
use slog::info;
use slog::warn;
use slog::Logger;
use slog_scope::GlobalLoggerGuard;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_failure::format_fail;
use replicante_util_tracing::tracer;
use replicante_util_upkeep::Upkeep;

use crate::actions;
use crate::api;
use crate::config::Agent as Config;
use crate::config::SentryConfig;
use crate::metrics::UPDATE_AVAILABLE;
use crate::Agent;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

/// Configure a command line parser.
///
/// The parser is configure with all the arguments every agent is required to implement.
/// Additional arguments can be added by each agent if needed.
pub fn clap<S1, S2, S3, S4>(
    name: S1,
    version: S2,
    description: S3,
    default_config_location: S4,
) -> Command
where
    S1: Into<clap::builder::Str>,
    S2: Into<clap::builder::Str>,
    S3: Into<clap::builder::StyledStr>,
    S4: Into<clap::builder::OsStr>,
{
    Command::new(name).version(version).about(description).arg(
        Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .num_args(1)
            .default_value(default_config_location)
            .value_parser(clap::value_parser!(String))
            .help("Specifies the configuration file to use"),
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
        .map_err(crate::AnyWrap::from)
        .with_context(|_| ErrorKind::Initialisation("tracer configuration failed".into()))?;

    let mut context = AgentContext::new(config, logger.clone(), tracer)?;
    register_process_metrics(&context);
    super::register_metrics(&context);
    context.store.migrate()?;
    let agent = initialise(&context, &mut upkeep)?;
    actions::initialise(&agent, &mut context, &mut upkeep)?;
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
            eprintln!("{}", message);
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
        // TODO: Fix error capturing after failure crate is removed.
        let hack = anyhow::anyhow!(error.to_string());
        sentry::integrations::anyhow::capture_anyhow(&hack);
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
    let options = sentry::ClientOptions {
        attach_stacktrace: true,
        dsn,
        in_app_include: vec!["replicante", "replicante_agent", "repliagent", "replisdk"],
        release: Some(release),
        ..Default::default()
    };
    let client = sentry::init(options);
    Ok(client)
}

/// Check for available updates in the background.
///
/// The check is performed once in a background thread that is ignored to avoid
/// startup or shutdown delays.
///
/// The check is only performed if the `update_checker` config option is set to true.
///
/// The result of the update, including any error, is reported in the logs.
/// If updates are available the `repliagent_upgradable` metric is also set to `1`.
pub fn update_checker(current: Version, url: &'static str, context: &AgentContext) -> Result<()> {
    if !context.config.update_checker {
        debug!(
            &context.logger,
            "Update checker is disabled, skipping check"
        );
        return Ok(());
    }
    let logger = context.logger.clone();
    Builder::new("r:b:update_checker")
        .full_name("replicante:base:update_checker")
        .spawn(move |scope| {
            let _activity = scope.scoped_activity("checking for updates");
            let response = match reqwest::blocking::get(url) {
                Ok(response) => response,
                Err(error) => {
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to fetch latest version information";
                        failure_info(&error)
                    );
                    return;
                }
            };
            let response = match response.json::<VersionMeta>() {
                Ok(response) => response,
                Err(error) => {
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to fetch latest version information";
                        failure_info(&error)
                    );
                    return;
                }
            };
            let latest = match Version::parse(&response.version) {
                Ok(version) => version,
                Err(error) => {
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to parse latest version information";
                        failure_info(&error)
                    );
                    return;
                }
            };
            if current < latest {
                UPDATE_AVAILABLE.set(1.0);
                warn!(
                    logger,
                    "A new version is available";
                    "current" => %current,
                    "latest" => %latest,
                );
                sentry::capture_event(sentry::protocol::Event {
                    level: sentry::Level::Warning,
                    message: Some("A new version is available".into()),
                    extra: {
                        let mut extra = BTreeMap::new();
                        extra.insert("current".into(), current.to_string().into());
                        extra.insert("latest".into(), latest.to_string().into());
                        extra
                    },
                    ..Default::default()
                });
            }
        })
        .with_context(|_| ErrorKind::ThreadSpawn("update_checker"))?;
    Ok(())
}

/// Version metadata returned by the server.
#[derive(Debug, Deserialize)]
struct VersionMeta {
    version: String,
}
