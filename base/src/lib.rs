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
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;
extern crate reqwest;
extern crate router;
extern crate semver;
extern crate sentry;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;

extern crate replicante_logging;
extern crate replicante_models_agent;
extern crate replicante_util_failure;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;
extern crate replicante_util_upkeep;

pub use semver::Version as SemVersion;

pub use replicante_util_tracing::fail_span;

mod api;
mod context;
mod error;
mod metrics;
mod traits;
mod versioned;

pub mod config;
pub mod process;

#[cfg(debug_assertions)]
pub mod testing;

pub use self::context::AgentContext;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::traits::Agent;
pub use self::versioned::ActiveAgent;
pub use self::versioned::AgentFactory;
pub use self::versioned::VersionedAgent;
