use std::fmt;
use std::sync::Arc;

#[cfg(debug_assertions)]
use std::time::Duration;

use iron::Iron;
use opentracingrust::Tracer;
use prometheus::Registry;
use prometheus::process_collector::ProcessCollector;

#[cfg(debug_assertions)]
use slog::Discard;
use slog::Logger;
use slog_scope::GlobalLoggerGuard;

#[cfg(debug_assertions)]
use replicante_util_tracing::TracerExtra;

use super::config::Agent as AgentConfig;
use super::api;
use super::Agent;


/// Agent services injection.
///
/// A container to allow agents and the agent runner to access configured
/// sub-systems like logging, metrics, etc ...
// Cannot derive Debug because `Tracer` :-(
// Any new field must be added to the implementation of Debug.
#[derive(Clone)]
pub struct AgentContext {
    pub config: AgentConfig,
    pub logger: Logger,

    /// Acess the agent's metrics [`Registry`].
    ///
    /// Agents MUST register their metrics at creation time and as part of the same [`Registry`].
    ///
    /// [`Registry`]: https://docs.rs/prometheus/0.3.13/prometheus/struct.Registry.html
    pub metrics: Registry,

    /// Access the agent's [`Tracer`].
    ///
    /// This is the agent's way to access the opentracing compatible tracer.
    ///
    /// [`Tracer`]: https://docs.rs/opentracingrust/0.3.0/opentracingrust/struct.Tracer.html
    pub tracer: Arc<Tracer>,
}

impl fmt::Debug for AgentContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "AgentContext{{config:{:?},logger:{:?},metrics:Registry,tracer:Tracer}}",
            self.config, self.logger,
        )
    }
}

impl AgentContext {
    pub fn new(config: AgentConfig, logger: Logger, tracer: Tracer) -> AgentContext {
        let metrics = Registry::new();
        AgentContext {
            config,
            logger,
            metrics,
            tracer: Arc::new(tracer),
        }
    }

    /// Configure and instantiate the logger.
    pub fn logger(config: &AgentConfig) -> (Logger, GlobalLoggerGuard) {
        let logger_opts = ::replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
        let logger = ::replicante_logging::configure(config.logging.clone(), &logger_opts);
        let scope_guard = slog_scope::set_global_logger(logger.clone());
        slog_stdlog::init().expect("Failed to initialise log -> slog integration");
        (logger, scope_guard)
    }

    #[cfg(debug_assertions)]
    pub fn mock() -> (AgentContext, TracerExtra) {
        let config = AgentConfig::default();
        let logger = Logger::root(Discard, o!());
        let (tracer, mut extra) = ::replicante_util_tracing::tracer(
            ::replicante_util_tracing::Config::Noop, logger.clone()
        ).unwrap();
        let context = AgentContext::new(config, logger, tracer);
        if let TracerExtra::ReporterThread(ref mut reporter) = extra {
            reporter.stop_delay(Duration::from_millis(2));
        };
        (context, extra)
    }
}


/// Common implementation for Agents.
///
/// This runner implements common logic that every
/// agent will need on top of the `Agent` trait.
pub struct AgentRunner {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl AgentRunner {
    pub fn new<A>(agent: A, context: AgentContext) -> AgentRunner
        where A: 'static + Agent
    {
        AgentRunner {
            agent: Arc::new(agent),
            context,
        }
    }

    /// Register all static metrics provided by the base agent.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        let process = ProcessCollector::for_self();
        if let Err(error) = registry.register(Box::new(process)) {
            debug!(logger, "Failed to register process metrics"; "error" => ?error);
        }
        api::register_metrics(logger, registry);
    }

    /// Starts the Agent process and waits for it to terminate.
    ///
    /// # Panics
    ///
    /// This method panics if:
    ///
    ///   * It fails to bind to the configured port.
    pub fn run(&self) {
        let bind = &self.context.config.api.bind;
        let handler = api::mount(Arc::clone(&self.agent), self.context.clone());
        info!(self.context.logger, "Agent API ready"; "bind" => bind);
        Iron::new(handler)
            .http(bind)
            .expect("Unable to start server");
    }
}
