Releasing agents
================
Agents release steps:

- [ ] Release `common` crates:
  - [ ] Bump version numbers as needed
  - [ ] Commit changes if needed
  - [ ] Release crates below
  - [ ] Update subrepo and versions in agents
- [ ] Ensure dependences are up to date:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Ensure tests and CI checks pass
- [ ] Bump the version number of all crates that need it
- [ ] Update changelog with version and date
- [ ] Update cargo lock file
- [ ] Ensure docker image builds correctly:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Git commit release
- [ ] Validate replicante_agent create (cargo package)
- [ ] Git tag release
- [ ] Build and push docker images:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Publish base cargo crate
- [ ] Release pre-built binaries


Publishing the base agent crate
===============================
In order for the `replicante_agent` crate to be published the following,
otherwise internal, crates need to be publised as well:

- [ ] replicante_logging
- [ ] replicante_models_agent
- [ ] replicante_util_failure
- [ ] replicante_util_upkeep
- [ ] replicante_util_tracing
- [ ] replicante_util_actixweb
