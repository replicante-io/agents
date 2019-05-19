//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
#![doc(html_root_url = "https://docs.rs/replicante_agent/0.3.0")]
extern crate clap;
extern crate failure;
extern crate failure_derive;
extern crate humthreads;
extern crate iron;
extern crate iron_json_response;
#[cfg(test)]
extern crate iron_test;
extern crate router;
#[macro_use]
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;
extern crate sentry;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;

extern crate replicante_agent_models;
extern crate replicante_logging;
extern crate replicante_util_failure;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;
extern crate replicante_util_upkeep;

mod api;
mod context;
mod error;
mod traits;
mod versioned;

pub mod config;
pub mod process;
pub mod util;

#[cfg(debug_assertions)]
pub mod testing;

pub use self::context::AgentContext;
pub use self::error::fail_span;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::traits::Agent;
pub use self::versioned::ActiveAgent;
pub use self::versioned::AgentFactory;
pub use self::versioned::VersionedAgent;

/// Register all base agent metrics.
pub fn register_metrics(context: &AgentContext) {
    api::register_metrics(context);
}
