use std::sync::Arc;

use iron::Chain;
use iron::Iron;
use router::Router;
use prometheus::process_collector::ProcessCollector;

use replicante_util_iron::MetricsHandler;
use replicante_util_iron::MetricsMiddleware;

use slog::Discard;
use slog::Logger;

use super::api;
use super::config;

use super::Agent;


/// Container type to hold an Agent trait object.
///
/// This type also adds the Send and Sync requirements needed by the
/// API handlers to hold a reference to an Agent implementation.
pub type AgentContainer = Arc<Agent>;


/// Common implementation for Agents.
///
/// This runner implements common logic that every
/// agent will need on top of the `Agent` trait.
pub struct AgentRunner {
    agent: AgentContainer,
    conf: config::AgentConfig,
}

impl AgentRunner {
    pub fn new<A>(agent: A, conf: config::AgentConfig) -> AgentRunner
        where A: 'static + Agent
    {
        AgentRunner { agent: Arc::new(agent), conf }
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
        let agent_info = api::AgentInfo::new(Arc::clone(&self.agent));
        let datastore_info = api::DatastoreInfo::new(Arc::clone(&self.agent));
        let metrics = MetricsHandler::new(self.agent.metrics().clone());
        let status = api::Shards::new(Arc::clone(&self.agent));

        router.get("/", api::index, "index");
        router.get("/api/v1/info/agent", agent_info, "agent_info");
        router.get("/api/v1/info/datastore", datastore_info, "datastore_info");
        router.get("/api/v1/metrics", metrics, "metrics");
        router.get("/api/v1/status", status, "status");

        // Setup metrics collection.
        let registry = self.agent.metrics();
        let (duration, errors, requests) = MetricsMiddleware::metrics("replicante_agent");
        registry.register(Box::new(duration.clone()))
            .expect("Unable to register duration histogram");
        registry.register(Box::new(errors.clone())).expect("Unable to register errors counter");
        registry.register(Box::new(requests.clone()))
            .expect("Unable to register requests counter");

        // Setup process metrics.
        let process = ProcessCollector::for_self();
        registry.register(Box::new(process)).expect("Unable to register process metrics");

        // TODO: setup logging properly.
        let logger = Logger::root(Discard, o!());

        // Wrap the router with middleweres.
        let metrics = MetricsMiddleware::new(duration, errors, requests, logger);
        let mut handler = Chain::new(router);
        handler.link(metrics.into_middleware());

        // Start the agent server.
        let bind = &self.conf.server.bind;
        println!("Listening on {} ...", bind);
        Iron::new(handler)
            .http(bind)
            .expect("Unable to start server");
    }
}
