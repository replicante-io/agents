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
//!
//! extern crate replicante_agent;
//! extern crate replicante_agent_models;
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
//! use replicante_agent_models::AgentVersion;
//! use replicante_agent_models::DatastoreInfo;
//! use replicante_agent_models::Shard;
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
//!     fn datastore_info(&self, _: &mut Span) -> AgentResult<DatastoreInfo> {
//!         Ok(DatastoreInfo::new("Cluster", "Test DB", "Test", "1.2.3"))
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
//!         TestAgent::new(tracer),
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
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate replicante_agent_models;
extern crate replicante_util_iron;

#[macro_use]
extern crate slog;


mod api;
mod runner;
mod traits;

pub mod config;
pub mod error;
pub mod util;

#[cfg(test)]
pub mod testing;

pub use self::error::AgentError;
pub use self::error::AgentResult;
pub use self::runner::AgentRunner;
pub use self::traits::Agent;
