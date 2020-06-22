//! This crate provides interfaces and structs to build Replicante agents.
//!
//! The crate provides a base `Agent` trait defining a common interface.
#![doc(html_root_url = "https://docs.rs/replicante_agent/0.6.0")]
pub use semver::Version as SemVersion;

pub use replicante_util_tracing::fail_span;

pub mod actions;
mod api;
mod context;
mod error;
mod metrics;
mod store;
mod traits;
mod versioned;

pub mod config;
pub mod process;

#[cfg(any(test, feature = "with_test_support"))]
pub mod testing;

pub use self::context::AgentContext;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::store::Transaction;
pub use self::traits::Agent;
pub use self::versioned::ActiveAgent;
pub use self::versioned::AgentFactory;
pub use self::versioned::VersionedAgent;
