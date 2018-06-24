Releasing agents
================
Agents release steps:

- [ ] Ensure tests and CI checks pass
- [ ] Bump the version number of all crates that need it
- [ ] Update changelogs with version and date
- [ ] Update docs version metadata
- [ ] Git commit and tag release
- [ ] Publish base cargo crate
- [ ] Release documentation


Publishing the base agent crate
===============================
In order for the `replicante_agent` crate to be published the following,
otherwise internal, crates need to be publised as well:

- [ ] replicante_agent_models
- [ ] replicante_util_iron
