use std::sync::Arc;

use iron::Chain;
use iron::Iron;
use router::Router;
use prometheus::process_collector::ProcessCollector;

use replicante_util_iron::MetricsHandler;
use replicante_util_iron::MetricsMiddleware;
use replicante_util_iron::RequestLogger;

use slog::Logger;

use super::api;
use super::config::Agent as AgentConfig;

use super::Agent;


/// Agent services injection.
///
/// A container to allow agents and the agent runner to access configured
/// sub-systems like logging, metrics, etc ...
#[derive(Clone, Debug)]
pub struct AgentContext {
    pub config: AgentConfig,
    pub logger: Logger,
}

impl AgentContext {
    pub fn new(config: AgentConfig) -> AgentContext {
        let logger_opts = ::replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
        let logger = ::replicante_logging::configure(config.logging.clone(), &logger_opts);
        AgentContext {
            config,
            logger,
        }
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
        let metrics = MetricsHandler::new(self.agent.metrics().clone());
        let shards = api::Shards::make(Arc::clone(&self.agent), self.context.clone());

        router.get("/", api::index, "index");
        router.get("/api/v1/info/agent", agent_info, "agent_info");
        router.get("/api/v1/info/datastore", datastore_info, "datastore_info");
        router.get("/api/v1/metrics", metrics, "metrics");
        router.get("/api/v1/shards", shards, "shards");

        // Setup metrics collection.
        let registry = self.agent.metrics();
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
