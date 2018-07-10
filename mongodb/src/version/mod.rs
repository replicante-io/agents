use std::sync::Arc;

use mongodb::Client;
use mongodb::ClientOptions;
use mongodb::ThreadedClient;

use replicante_agent::ActiveAgent;
use replicante_agent::AgentContext;
use replicante_agent::AgentFactory;
use replicante_agent::Result;

use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;

use super::config::Config;
use super::errors;


pub mod v3_2;


lazy_static! {
    static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// An `AgentFactory` that returns a MongoDB 3.2+ Replica Set compatible agent.
pub struct MongoDBFactory {
    client: Client,
    context: AgentContext,
}

impl MongoDBFactory {
    pub fn new(config: Config, context: AgentContext) -> Result<MongoDBFactory> {
        let mut options = ClientOptions::default();
        options.server_selection_timeout_ms = config.mongo.timeout;
        let client = Client::with_uri_and_options(&config.mongo.uri, options)
            .map_err(errors::to_agent)?;
        Ok(MongoDBFactory {
            client,
            context,
        })
    }
}

impl AgentFactory for MongoDBFactory {
    fn make(&self) -> ActiveAgent {
        let agent = v3_2::ReplicaSet::new(self.client.clone(), self.context.clone());
        let agent = Arc::new(agent);
        ActiveAgent::new(agent, false, "v3.2")
    }

    fn should_remake(&self, _: &ActiveAgent, _: &DatastoreInfo) -> bool {
        false
    }
}
