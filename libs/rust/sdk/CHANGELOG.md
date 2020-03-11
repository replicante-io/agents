# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Optional `ActionHook`s to provide store-specific implementation of SDK actions.

### Changed
- **BREAKING**: Removed Agent method to expose the graceful stop action.
- **BREAKING**: Removed Agent method to expose a default service name.
- **BREAKING**: Rework action kinds format.
- Service actions are disabled unless a service configuration is provided.
- Service configuration is optional and skipped by default.

## [0.4.1] - 2020-03-07
### Added
- Action definition.
- Action engine.
- Actions register.
- API over HTTPS with optional client authentication.
- DevTool: TLS certificates generation scripts.
- Initial actions API.
- Persistent DB with support for migrations.
- Service related actions.

### Changed
- Replace Iron web framework with Actix Web.

## [0.4.0] - 2019-06-16
### Added
- Graceful shutdown.
- Introduce an `/api/unstable` API "version".
- Sentry integration.
- Threads introspection API.
- Update checker helper function.
- `cluster_display_name` override option.
- `process` module to reduce agent process duplication.

### Changed
- **BREAKING**: Rename incorrectly named v1 API as unstable.
- **BREAKING**: Rename model fields to match spec.
- **BREAKING**: Replaced `AgentRunner` with `::replicante_agent::process::run`.
- Improved API tracing framework.

## [0.3.0] - 2019-03-29
### Changed
- **BREAKING**: Replace `error-chain` with `failure`.
- Standardise API error response.
- Standardise logging across core and agents.

## [0.2.0] - 2019-02-20
### Added
- Logging support
- Versioned agents utilities

### Changed
- **BREAKING**: Move metrics from `Agent` trait to `AgentContext`
- **BREAKING**: Move tracing from `Agent` trait to `AgentContext`
- **BREAKING**: Move span receive/send logs to base agent.
- **BREAKING**: Rework configuration using [serde](https://docs.rs/serde)
- **BREAKING**: Update agent models to match latest specs
- **BREAKING**: Use [error_chain](https://docs.rs/error-chain) for errors
- Refactor metrics into lazy statics and registration methods

## 0.1.0 - 2018-06-28
### Added
- Agent metrics
- Base Agent traits and structs
- OpenTracing integration


[Unreleased]: https://github.com/replicante-io/agents/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/replicante-io/agents/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/replicante-io/agents/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/replicante-io/agents/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/replicante-io/agents/compare/v0.1.0...v0.2.0
