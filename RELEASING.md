Releasing agents
================
Agents release steps:

- [ ] Ensure dependences are up to date:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Ensure tests and CI checks pass
- [ ] Release `common` crates:
  - [ ] Bump version numbers as needed
  - [ ] Commit changes if needed
  - [ ] Release in reverse order below
  - [ ] Update subrepo and versions in agents
- [ ] Bump the version number of all crates that need it
- [ ] Update cargo lock file
- [ ] Update changelog with version and date
- [ ] Version documents
- [ ] Ensure docker image builds correctly:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Git commit and tag release
- [ ] Build and push docker images:
  - [ ] For the main workspace
  - [ ] For the Kafka workspace
- [ ] Publish base cargo crate
- [ ] Release pre-built binaries
- [ ] Release documentation


Publishing the base agent crate
===============================
In order for the `replicante_agent` crate to be published the following,
otherwise internal, crates need to be publised as well:

- [ ] replicante_agent_models
- [ ] replicante_logging
- [ ] replicante_util_failure
- [ ] replicante_util_iron
- [ ] replicante_util_tracing
- [ ] replicante_util_upkeep
