//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
//!
//! To create an agent implement the `Agent` trait for a struct and pass that
//! struct to `AgentRunner::new` to create a runner.
//! The `AgentRunner::run` method will then spin up the API server.
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
