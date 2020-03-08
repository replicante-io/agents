# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2020-03-07
### Added
- Actions system.

### Added
- Support graceful shutdown.

## [0.4.0] - 2019-06-16
### Changed
- **BREAKING**: Upgrade base agent library to latest.

## [0.3.0] - 2019-03-29
### Changed
- **BREAKING**: Replace `error-chain` with `failure`.

## [0.2.0] - 2019-02-20
### Added
- Command line `--config` option
- Logging support
- Support MongoDB 3.0 (replica set)
- Support MongoDB 3.2+ (sharded mongo)
- Timeout configuration option

### Changed
- **BREAKING**: Rename MongoDB Agent binary
- **BREAKING**: Rework configuration using [serde](https://docs.rs/serde)
- **BREAKING**: Update agent models to match latest specs
- **BREAKING**: Use [error_chain](https://docs.rs/error-chain) for errors
- Move MongoDB interactions into a trait

## 0.1.0 - 2018-06-28
### Added
- Agent metrics
- Basic info and sharding data for MongoDB 3.2+
- OpenTracing integration


[Unreleased]: https://github.com/replicante-io/agents/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/replicante-io/agents/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/replicante-io/agents/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/replicante-io/agents/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/replicante-io/agents/compare/v0.1.0...v0.2.0
