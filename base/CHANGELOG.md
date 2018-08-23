# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Logging support
- Versioned agents utilities

### Changed
- **BREAKING**: Move metrics from `Agent` trait to `AgentContext`
- **BREAKING**: Move tracing from `Agent` trait to `AgentContext`
- **BREAKING**: Rework configuration using [serde](https://docs.rs/serde)
- **BREAKING**: Update agent models to match latest specs
- **BREAKING**: Use [error_chain](https://docs.rs/error-chain) for errors
- Refactor metrics into lazy statics and registration methods

## 0.1.0 - 2018-06-28
### Added
- Agent metrics
- Base Agent traits and structs
- OpenTracing integration


[Unreleased]: https://github.com/replicante-io/agents/compare/v0.1.0...HEAD
