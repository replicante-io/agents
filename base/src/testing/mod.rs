use opentracingrust::Span;

use super::Agent;
use super::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;


/// An implementation of Agent to be used for tests.
pub struct MockAgent {
    pub agent_info: ::std::result::Result<AgentInfo, String>,
    pub datastore_info: ::std::result::Result<DatastoreInfo, String>,
    pub shards: ::std::result::Result<Shards, String>,
}

impl MockAgent {
    pub fn new() -> MockAgent {
        let agent_info = Ok(AgentInfo::new(AgentVersion::new("dcd", "1.2.3", "tainted")));
        let datastore_info = Ok(DatastoreInfo::new("cluster", "DB", "mock", "1.2.3"));
        let shards = Ok(Shards::new(vec![]));
        MockAgent {
            agent_info,
            datastore_info,
            shards,
        }
    }
}

impl Agent for MockAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        self.agent_info.clone().map_err(|err| err.into())
    }

    fn datastore_info(&self, _: &mut Span) -> Result<DatastoreInfo> {
        self.datastore_info.clone().map_err(|err| err.into())
    }

    fn shards(&self, _:&mut Span) -> Result<Shards> {
        self.shards.clone().map_err(|err| err.into())
    }
}
