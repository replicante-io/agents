//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
//!
//! To create an agent implement the `Agent` trait for a struct and pass that
//! struct to `AgentRunner::new` to create a runner.
//! The `AgentRunner::run` method will then spin up the API server.
#![doc(html_root_url = "https://docs.rs/replicante_agent/0.3.0")]
extern crate failure;
extern crate failure_derive;
extern crate humthreads;
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
extern crate replicante_util_failure;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;

#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;

mod api;
mod error;
mod runner;
mod traits;
mod versioned;

pub mod config;
pub mod util;

#[cfg(debug_assertions)]
pub mod testing;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::error::fail_span;

pub use self::runner::AgentContext;
pub use self::runner::AgentRunner;
pub use self::traits::Agent;
pub use self::versioned::ActiveAgent;
pub use self::versioned::AgentFactory;
pub use self::versioned::VersionedAgent;
