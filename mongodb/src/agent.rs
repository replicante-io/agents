use std::sync::Arc;

use mongodb::Client;
use mongodb::ThreadedClient;

use opentracingrust::Log;
use opentracingrust::Span;

use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::errors;
use super::version;

use super::config::Config;
use super::version::MongoDBInterface;


/// Agent dealing with MongoDB 3.2+ Replica Sets.
pub struct MongoDBAgent {
    client: Client,

    // The interface is picked based on the MongoDB version.
    // A client is needed to determine the interface.
    //  - How to deal with starting without the datastore?
    //  - When is the interface replacement triggered?
    interface: Arc<MongoDBInterface>,
}

impl MongoDBAgent {
    pub fn new(config: Config, context: AgentContext) -> Result<MongoDBAgent> {
        let client = Client::with_uri(&config.mongo.uri)
            .map_err(errors::to_agent)?;
        // Start off with a reasonable default version of 3.2.
        // Let the automated version detection figure out the exact interface.
        let interface = Arc::new(version::v3_2::ReplicaSet::new(context.clone()));
        Ok(MongoDBAgent {
            client,
            interface,
        })
    }
}

impl Agent for MongoDBAgent {
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        span.log(Log::new().log("span.kind", "server-receive"));
        let version = AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
        );
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(AgentInfo::new(version))
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        self.interface.datastore_info(span, &self.client)
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        self.interface.shards(span, &self.client)
    }
}
