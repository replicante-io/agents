# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- **BREAKING**: Rename binary from `replicante-agent-kafka` to `repliagent-kafka`.

## [0.5.0] - 2020-05-28
### Changed
- **BREAKING**: Rework action kinds format.

## [0.4.1] - 2020-03-07
### Added
- Actions system.

## [0.4.0] - 2019-06-16
### Changed
- **BREAKING**: Upgrade base agent library to latest.

## [0.3.0] - 2019-03-29
## Changed
- **BREAKING**: Replace `error-chain` with `failure`.
- Reduce kafka agent docker image size.

## [0.2.0] - 2019-02-20
### Changed
- **BREAKING**: Rework configuration using [serde](https://docs.rs/serde)
- **BREAKING**: Update agent models to match latest specs

## 0.1.0
### Added
- Initial kafka agent


[Unreleased]: https://github.com/replicante-io/agents/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/replicante-io/agents/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/replicante-io/agents/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/replicante-io/agents/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/replicante-io/agents/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/replicante-io/agents/compare/v0.1.0...v0.2.0
