//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
//!
//! To create an agent implement the `Agent` trait for a struct and pass that
//! struct to `AgentRunner::new` to create a runner.
//! The `AgentRunner::run` method will then spin up the API server.
//!
//! # Examples
//!
//! ```
//! extern crate opentracingrust;
//! extern crate prometheus;
//! extern crate replicante_agent;
//!
//! use opentracingrust::Span;
//! use opentracingrust::Tracer;
//! use opentracingrust::tracers::NoopTracer;
//! use prometheus::Registry;
//!
//! use replicante_agent::Agent;
//! use replicante_agent::AgentResult;
//! use replicante_agent::AgentRunner;
//!
//! use replicante_agent::config::AgentConfig;
//!
//! use replicante_agent::models::AgentVersion;
//! use replicante_agent::models::DatastoreVersion;
//! use replicante_agent::models::Shard;
//! 
//! 
//! pub struct TestAgent {
//!     registry: Registry,
//!     tracer: Tracer,
//! }
//! 
//! impl TestAgent {
//!     pub fn new(tracer: Tracer) -> TestAgent {
//!         TestAgent {
//!             registry: Registry::new(),
//!             tracer,
//!         }
//!     }
//! }
//! 
//! impl Agent for TestAgent {
//!     fn agent_version(&self, _: &mut Span) -> AgentResult<AgentVersion> {
//!         Ok(AgentVersion::new(
//!             env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
//!             env!("GIT_BUILD_TAINT")
//!         ))
//!     }
//!
//!     fn datastore_version(&self, _: &mut Span) -> AgentResult<DatastoreVersion> {
//!         Ok(DatastoreVersion::new("Test DB", "1.2.3"))
//!     }
//!
//!     fn shards(&self, _: &mut Span) -> AgentResult<Vec<Shard>> {
//!         Ok(vec![])
//!     }
//!
//!     fn metrics(&self) -> Registry {
//!         self.registry.clone()
//!     }
//!
//!     fn tracer(&self) -> &Tracer {
//!         &self.tracer
//!     }
//! }
//! 
//! 
//! fn main() {
//!     let (tracer, _receiver) = NoopTracer::new();
//!     let runner = AgentRunner::new(
//!         Box::new(TestAgent::new(tracer)),
//!         AgentConfig::default(),
//!     );
//!     // This will block the process serving requests.
//!     //runner.run();
//! }
//! ```
extern crate config as config_crate;

extern crate iron;
extern crate iron_json_response;
extern crate router;
#[cfg(test)]
extern crate iron_test;

extern crate opentracingrust;
extern crate opentracingrust_zipkin;
extern crate prometheus;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


use std::sync::Arc;

use iron::Iron;
use router::Router;

use opentracingrust::Span;
use opentracingrust::Tracer;
use prometheus::Registry;

mod api;
pub mod config;
pub mod error;
pub mod models;
pub mod util;

pub use self::error::AgentError;
pub use self::error::AgentResult;

use self::models::AgentVersion;
use self::models::DatastoreVersion;
use self::models::Shard;


/// Trait to share common agent code and features.
///
/// Agents should be implemented as structs that implement `BaseAgent`.
pub trait Agent : Send + Sync {
    //*** Methods to access datastore model requirements ***//
    /// Fetches the agent version information.
    fn agent_version(&self, span: &mut Span) -> AgentResult<AgentVersion>;

    /// Fetches the datastore version information.
    fn datastore_version(&self, span: &mut Span) -> AgentResult<DatastoreVersion>;

    /// Fetches all shards and details on the managed datastore node.
    fn shards(&self, span: &mut Span) -> AgentResult<Vec<Shard>>;


    //*** Methods needed for agent introspection and diagnostics ***//
    /// Acess the agent's metrics [`Registry`].
    ///
    /// Agents MUST register their metrics at creation time and as part of the same [`Registry`].
    ///
    /// [`Registry`]: https://docs.rs/prometheus/0.3.13/prometheus/struct.Registry.html
    fn metrics(&self) -> Registry;

    /// Access the agent's [`Tracer`].
    ///
    /// This is the agent's way to access the optional opentracing compatible tracer.
    ///
    /// [`Tracer`]: https://docs.rs/opentracingrust/0.3.0/opentracingrust/struct.Tracer.html
    fn tracer(&self) -> &Tracer;
}

/// Container type to hold an Agent trait object.
///
/// This type also adds the Send and Sync requirements needed by the
/// API handlers to hold a reference to an Agent implementation.
type AgentContainer = Arc<Box<Agent>>;


/// Common implementation for Agents.
///
/// This runner implements common logic that every
/// agent will need on top of the `Agent` trait.
pub struct AgentRunner {
    agent: AgentContainer,
    conf: self::config::AgentConfig,
}

impl AgentRunner {
    pub fn new(agent: Box<Agent>, conf: self::config::AgentConfig) -> AgentRunner {
        AgentRunner { agent: Arc::new(agent), conf }
    }

    /// Starts the Agent process and waits for it to terminate.
    pub fn run(&self) -> () {
        let mut router = Router::new();
        let info = api::InfoHandler::new(Arc::clone(&self.agent));
        let status = api::StatusHandler::new(Arc::clone(&self.agent));

        router.get("/", api::index, "index");
        router.get("/api/v1/info", info, "info");
        router.get("/api/v1/status", status, "status");

        let bind = &self.conf.server.bind;
        println!("Listening on {} ...", bind);
        Iron::new(router)
            .http(bind)
            .expect("Unable to start server");
    }
}
