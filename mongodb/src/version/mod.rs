use std::sync::Arc;

use error_chain::ChainedError;

use mongodb::Client;
use mongodb::ClientOptions;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use mongodb::topology::TopologyDescription;
use mongodb::topology::TopologyType;

use semver::Version;

use replicante_agent::ActiveAgent;
use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::AgentFactory;
use replicante_agent::Error;
use replicante_agent::Result;
use replicante_agent::ResultExt;
use replicante_agent_models::DatastoreInfo;

use super::config::Config;
use super::errors;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


mod common;
mod v3_0;
mod v3_2;


/// An `AgentFactory` that returns a MongoDB 3.2+ Replica Set compatible agent.
pub struct MongoDBFactory {
    client: Client,
    context: AgentContext,
}

impl MongoDBFactory {
    pub fn new(config: Config, context: AgentContext) -> Result<MongoDBFactory> {
        let mut options = ClientOptions::default();
        options.server_selection_timeout_ms = config.mongo.timeout;

        // Create a MongoDB client out of a URI but explicitly setting the topology
        // to single server to ensure the requested node is used.
        let uri = ::mongodb::connstring::parse(&config.mongo.uri)
            .chain_err(|| "Failed to parse mongo URI string")?;
        let mut description = TopologyDescription::new(options.stream_connector.clone());
        description.topology_type = TopologyType::Single;
        let client = Client::with_config(uri, Some(options), Some(description))
            .map_err(errors::to_agent)?;
        debug!(
            context.logger, "MongoDB client created";
            "uri" => &config.mongo.uri, "timeout" => &config.mongo.timeout
        );

        Ok(MongoDBFactory {
            client,
            context,
        })
    }
}

impl MongoDBFactory {
    /// Make an agent to be used when a version could not be detected.
    fn default_agent(&self) -> (Arc<Agent>, &'static str, &'static str) {
        let agent = v3_2::ReplicaSet::new(self.client.clone(), self.context.clone());
        let agent = Arc::new(agent);
        (agent, "3.2.0", "replica-set")
    }

    /// Fetch the currently running version of MongoDB.
    fn mongo_version(&self) -> Result<Version> {
        MONGODB_OPS_COUNT.with_label_values(&["version"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["version"]).start_timer();
        let version = self.client.db("test").version().map_err(|error| {
            MONGODB_OP_ERRORS_COUNT.with_label_values(&["version"]).inc();
            errors::to_agent(error)
        }).chain_err(|| Error::from("Failed to detect version"))?;
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
                let error = error.display_chain().to_string();
                warn!(
                    self.context.logger, "Could not detect MongoDB version, using default agent";
                    "agent_version" => agent_version, "error" => error, "mode" => mode
                );
                ActiveAgent::new(agent, true, "unknown")
            },
            Ok(version) => {
                let agent = self.make_rs(&version);
                let mode = "replica-set";
                agent.map(|(agent, agent_version)| {
                    info!(
                        self.context.logger, "Instantiated MongoDB agent";
                        "agent_version" => agent_version, "mongo_version" => %version,
                        "mode" => mode
                    );
                    ActiveAgent::new(agent, false, version.to_string())

                // Failed to find a compatible version.
                }).unwrap_or_else(|| {
                    let (agent, agent_version, mode) = self.default_agent();
                    warn!(
                        self.context.logger, "Unsupported MongoDB version, using default agent";
                        "agent_version" => agent_version, "mongo_version" => %version,
                        "mode" => mode
                    );
                    ActiveAgent::new(agent, true, "unknown")
                })
            }
        }
    }

    /// Make a replica-set compatible agent, if versions allow it.
    fn make_rs(&self, version: &Version) -> Option<(Arc<Agent>, &'static str)> {
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
}


#[cfg(test)]
mod tests {
    use semver::Version;

    use replicante_agent::AgentContext;
    use replicante_agent::AgentFactory;
    use replicante_agent_models::DatastoreInfo;

    use super::Config;
    use super::MongoDBFactory;


    #[test]
    fn make_from_error() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Err("test on error".into()));
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(active.remake_on_error());
        assert_eq!(active.version_id(), "unknown");
    }

    #[test]
    fn make_from_version_above_32() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(!active.remake_on_error());
        assert_eq!(active.version_id(), "3.3.0");
    }

    #[test]
    fn make_from_version_exact_32() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let version = Version::parse("3.2.0").unwrap();
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(!active.remake_on_error());
        assert_eq!(active.version_id(), "3.2.0");
    }

    #[test]
    fn should_always_remake_unknown_version() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "unknown");
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Err("test".into()));
        let remake = factory.should_remake(&active, &info);
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(remake);
    }

    #[test]
    fn should_remake_changed_version() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "3.6.0");
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let remake = factory.should_remake(&active, &info);
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(remake);
    }

    #[test]
    fn should_remake_same_version() {
        let (context, extra) = AgentContext::mock();
        let config = Config::default();
        let info = DatastoreInfo::new("test", "MongoDB", "name", "3.3.0");
        let version = Version::parse("3.3.0").unwrap();
        let factory = MongoDBFactory::new(config, context).unwrap();
        let active = factory.make_agent(Ok(version));
        let remake = factory.should_remake(&active, &info);
        // Drop tracer before assertions to that panics don't lead to thread errors.
        drop(factory);
        drop(extra);
        assert!(!remake);
    }
}
