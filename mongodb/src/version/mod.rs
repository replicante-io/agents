use std::sync::Arc;

use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::topology::TopologyDescription;
use mongodb::topology::TopologyType;
use mongodb::Client;
use mongodb::ClientOptions;
use mongodb::ThreadedClient;
use semver::Version;
use slog::debug;
use slog::info;
use slog::warn;

use replicante_agent::ActiveAgent;
use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::AgentFactory;
use replicante_agent::Error;
use replicante_agent::Result;
use replicante_models_agent::DatastoreInfo;
use replicante_util_failure::failure_info;

use super::config::Config;
use super::config::Sharding;
use super::error::ErrorKind;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;

mod common;
mod v3_0;
mod v3_2;

const MONGODB_MODE_RS: &str = "replica-set";
const MONGODB_MODE_SHARDED: &str = "sharded-cluster";

/// An `AgentFactory` that returns a MongoDB 3.2+ Replica Set compatible agent.
pub struct MongoDBFactory {
    client: Client,
    context: AgentContext,
    sharded_mode: bool,
    sharding: Option<Sharding>,
}

impl MongoDBFactory {
    pub fn with_config(config: Config, context: AgentContext) -> Result<MongoDBFactory> {
        let mut options = ClientOptions::default();
        options.server_selection_timeout_ms = config.mongo.timeout;

        // Create a MongoDB client out of a URI but explicitly setting the topology
        // to single server to ensure the requested node is used.
        let uri = ::mongodb::connstring::parse(&config.mongo.uri)
            .with_context(|_| ErrorKind::ConfigOption("mongo.uri"))?;
        let mut description = TopologyDescription::new(options.stream_connector.clone());
        description.topology_type = TopologyType::Single;
        let client = Client::with_config(uri, Some(options), Some(description))
            .with_context(|_| ErrorKind::Connection("mongodb", config.mongo.uri.clone()))?;
        debug!(
            context.logger,
            "MongoDB client created";
            "uri" => &config.mongo.uri,
            "timeout" => &config.mongo.timeout,
        );

        let sharding = config.mongo.sharding;
        let sharded_mode = sharding.is_some() && sharding.as_ref().unwrap().enable;
        Ok(MongoDBFactory {
            client,
            context,
            sharded_mode,
            sharding,
        })
    }
}

impl MongoDBFactory {
    /// Make an agent to be used when a version could not be detected.
    fn default_agent(&self) -> (Arc<dyn Agent>, &'static str, &'static str) {
        if self.sharded_mode {
            let agent = v3_2::Sharded::new(
                self.sharding.as_ref().unwrap().clone(),
                self.client.clone(),
                self.context.clone(),
            );
            let agent = Arc::new(agent);
            (agent, "3.2.0", MONGODB_MODE_SHARDED)
        } else {
            let agent = v3_2::ReplicaSet::new(self.client.clone(), self.context.clone());
            let agent = Arc::new(agent);
            (agent, "3.2.0", MONGODB_MODE_RS)
        }
    }

    /// Fetch the currently running version of MongoDB.
    fn mongo_version(&self) -> Result<Version> {
        MONGODB_OPS_COUNT.with_label_values(&["version"]).inc();
        let timer = MONGODB_OPS_DURATION
            .with_label_values(&["version"])
            .start_timer();
        let version = self
            .client
            .db("test")
            .version()
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT
                    .with_label_values(&["version"])
                    .inc();
                error
            })
            .with_context(|_| ErrorKind::StoreOpFailed("version"))?;
        timer.observe_duration();
        Ok(version)
    }

    /// Instantiate a MongoDB agent based on the fetched version.
    ///
    /// If the version could not be determined returns a MongoDB 3.2 agent.
    fn make_agent(&self, version: Result<Version>) -> ActiveAgent {
        match version {
            Err(error) => {
                let (agent, agent_version, mode) = self.default_agent();
                warn!(
                    self.context.logger,
                    "Could not detect MongoDB version, using default agent";
                    "agent_version" => agent_version,
                    "mode" => mode,
                    failure_info(&error),
                );
                ActiveAgent::new(agent, "unknown")
            }
            Ok(version) => {
                let (agent, mode) = if self.sharded_mode {
                    (self.make_sharded(&version), MONGODB_MODE_SHARDED)
                } else {
                    (self.make_rs(&version), MONGODB_MODE_RS)
                };
                agent
                    .map(|(agent, agent_version)| {
                        info!(
                            self.context.logger,
                            "Instantiated MongoDB agent";
                            "agent_version" => agent_version,
                            "mongo_version" => %version,
                            "mode" => mode,
                        );
                        ActiveAgent::new(agent, version.to_string())
                    })
                    // Failed to find a compatible version.
                    .unwrap_or_else(|| {
                        let (agent, agent_version, mode) = self.default_agent();
                        warn!(
                            self.context.logger,
                            "Unsupported MongoDB version, using default agent";
                            "agent_version" => agent_version,
                            "mongo_version" => %version,
                            "mode" => mode,
                        );
                        ActiveAgent::new(agent, "unknown")
                    })
            }
        }
    }

    /// Make a replica-set compatible agent, if versions allow it.
    fn make_rs(&self, version: &Version) -> Option<(Arc<dyn Agent>, &'static str)> {
        if v3_2::REPLICA_SET_RANGE.matches(version) {
            let agent = v3_2::ReplicaSet::new(self.client.clone(), self.context.clone());
            Some((Arc::new(agent), "3.2.0"))
        } else if v3_0::REPLICA_SET_RANGE.matches(version) {
            let agent = v3_0::ReplicaSet::new(self.client.clone(), self.context.clone());
            Some((Arc::new(agent), "3.0.0"))
        } else {
            None
        }
    }

    /// Make a sharded-cluster compatible agent, if versions allow it.
    fn make_sharded(&self, version: &Version) -> Option<(Arc<dyn Agent>, &'static str)> {
        if v3_2::SHARDED_RANGE.matches(version) {
            let agent = v3_2::Sharded::new(
                self.sharding.as_ref().unwrap().clone(),
                self.client.clone(),
                self.context.clone(),
            );
            Some((Arc::new(agent), "3.2.0"))
        } else {
            None
        }
    }
}

impl AgentFactory for MongoDBFactory {
    fn make(&self) -> ActiveAgent {
        debug!(self.context.logger, "Instantiating a new MongoDB agent ...");
        let version = self.mongo_version();
        self.make_agent(version)
    }

    fn should_remake(&self, active: &ActiveAgent, info: &DatastoreInfo) -> bool {
        let version = active.version_id();
        version == "unknown" || *version != info.version
    }

    fn should_remake_on_error(&self, active: &ActiveAgent, _: &Error) -> bool {
        active.version_id() == "unknown"
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use replicante_agent::AgentContext;
    use replicante_agent::AgentFactory;
    use replicante_models_agent::DatastoreInfo;

    use super::Config;
    use super::ErrorKind;
    use super::MongoDBFactory;

    #[test]
    fn make_from_error() {
        let context = AgentContext::mock();
        let config = Config::default();
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Err(ErrorKind::MembersNoPrimary.into()));
        let error = ErrorKind::MembersNoPrimary.into();
        let remake_on_error = factory.should_remake_on_error(&active, &error);
        drop(factory);
        assert!(remake_on_error);
        assert_eq!(active.version_id(), "unknown");
    }

    #[test]
    fn make_from_version_above_32() {
        let context = AgentContext::mock();
        let config = Config::default();
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let error = ErrorKind::MembersNoPrimary.into();
        let remake_on_error = factory.should_remake_on_error(&active, &error);
        drop(factory);
        assert!(!remake_on_error);
        assert_eq!(active.version_id(), "3.3.0");
    }

    #[test]
    fn make_from_version_exact_32() {
        let context = AgentContext::mock();
        let config = Config::default();
        let version = Version::parse("3.2.0").unwrap();
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let error = ErrorKind::MembersNoPrimary.into();
        let remake_on_error = factory.should_remake_on_error(&active, &error);
        drop(factory);
        assert!(!remake_on_error);
        assert_eq!(active.version_id(), "3.2.0");
    }

    #[test]
    fn should_always_remake_unknown_version() {
        let context = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "unknown", None);
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Err(ErrorKind::MembersNoPrimary.into()));
        let remake = factory.should_remake(&active, &info);
        drop(factory);
        assert!(remake);
    }

    #[test]
    fn should_remake_changed_version() {
        let context = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "3.6.0", None);
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let remake = factory.should_remake(&active, &info);
        drop(factory);
        assert!(remake);
    }

    #[test]
    fn should_remake_same_version() {
        let context = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "3.3.0", None);
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::with_config(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let remake = factory.should_remake(&active, &info);
        drop(factory);
        assert!(!remake);
    }
}
