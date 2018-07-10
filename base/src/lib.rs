//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
//!
//! To create an agent implement the `Agent` trait for a struct and pass that
//! struct to `AgentRunner::new` to create a runner.
//! The `AgentRunner::run` method will then spin up the API server.
#![doc(html_root_url = "https://docs.rs/replicante_agent/0.2.0")]
#[macro_use]
extern crate error_chain;

extern crate iron;
extern crate iron_json_response;
extern crate router;
#[cfg(test)]
extern crate iron_test;

#[macro_use]
extern crate lazy_static;

extern crate opentracingrust;
extern crate opentracingrust_zipkin;
extern crate prometheus;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

extern crate replicante_agent_models;
extern crate replicante_logging;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;

#[macro_use]
extern crate slog;


mod api;
mod errors;
mod runner;
mod traits;
mod versioned;

pub mod config;
pub mod util;

#[cfg(debug_assertions)]
pub mod testing;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;

pub use self::runner::AgentContext;
pub use self::runner::AgentRunner;
pub use self::traits::Agent;
pub use self::versioned::AgentFactory;
pub use self::versioned::VersionedAgent;
