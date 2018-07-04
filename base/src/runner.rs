use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use std::time::Duration;

use iron::Chain;
use iron::Iron;
use router::Router;

use opentracingrust::Tracer;
use prometheus::Registry;
use prometheus::process_collector::ProcessCollector;

#[cfg(test)]
use slog::Discard;
use slog::Logger;

use replicante_util_iron::MetricsHandler;
use replicante_util_iron::MetricsMiddleware;
use replicante_util_iron::RequestLogger;

#[cfg(test)]
use replicante_util_tracing::TracerExtra;

use super::Agent;
use super::api;
use super::config::Agent as AgentConfig;


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
            f, "AgentContext{{config:{:?},logger:{:?},tracer:Tracer}}",
            self.config, self.logger
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
    pub fn logger(config: &AgentConfig) -> Logger {
        let logger_opts = ::replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
        ::replicante_logging::configure(config.logging.clone(), &logger_opts)
    }

    #[cfg(test)]
    pub fn mock() -> (AgentContext, TracerExtra) {
        let config = AgentConfig::default();
        let logger = Logger::root(Discard, o!());
        let (tracer, mut extra) = ::replicante_util_tracing::tracer(
            ::replicante_util_tracing::Config::Noop, logger.clone()
        ).unwrap();
        let context = AgentContext::new(config, logger, tracer);
        match extra {
            TracerExtra::ReporterThread(ref mut reporter) => {
                reporter.stop_delay(Duration::from_millis(2));
            },
            _ => ()
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

    /// Starts the Agent process and waits for it to terminate.
    ///
    /// # Panics
    ///
    /// This method panics if:
    ///
    ///   * It fails to configure or register the metrics.
    ///   * It fails to bind to the configured port.
    pub fn run(&self) -> () {
        // Create and configure API handlers.
        let mut router = Router::new();
        let agent_info = api::AgentInfo::make(Arc::clone(&self.agent), self.context.clone());
        let datastore_info = api::DatastoreInfo::make(
            Arc::clone(&self.agent), self.context.clone()
        );
        let metrics = MetricsHandler::new(self.context.metrics.clone());
        let shards = api::Shards::make(Arc::clone(&self.agent), self.context.clone());

        router.get("/", api::index, "index");
        router.get("/api/v1/info/agent", agent_info, "agent_info");
        router.get("/api/v1/info/datastore", datastore_info, "datastore_info");
        router.get("/api/v1/metrics", metrics, "metrics");
        router.get("/api/v1/shards", shards, "shards");

        // Setup metrics collection.
        let registry = &self.context.metrics;
        let (duration, errors, requests) = MetricsMiddleware::metrics("replicante_agent");
        registry.register(Box::new(duration.clone()))
            .expect("Unable to register duration histogram");
        registry.register(Box::new(errors.clone())).expect("Unable to register errors counter");
        registry.register(Box::new(requests.clone()))
            .expect("Unable to register requests counter");
        let metrics = MetricsMiddleware::new(
            duration, errors, requests, self.context.logger.clone()
        );

        // Setup process metrics.
        let process = ProcessCollector::for_self();
        registry.register(Box::new(process)).expect("Unable to register process metrics");

        // Wrap the router with middleweres.
        let mut handler = Chain::new(router);
        handler.link_after(RequestLogger::new(self.context.logger.clone()));
        handler.link(metrics.into_middleware());

        // Start the agent server.
        let bind = &self.context.config.api.bind;
        info!(self.context.logger, "Agent API ready"; "bind" => bind);
        Iron::new(handler)
            .http(bind)
            .expect("Unable to start server");
    }
}
