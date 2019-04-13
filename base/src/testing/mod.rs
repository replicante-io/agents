use opentracingrust::Span;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::Agent;
use super::ErrorKind;
use super::Result;

/// An implementation of Agent to be used for tests.
pub struct MockAgent {
    pub agent_info: ::std::result::Result<AgentInfo, String>,
    pub datastore_info: ::std::result::Result<DatastoreInfo, String>,
    pub shards: ::std::result::Result<Shards, String>,
}

impl MockAgent {
    pub fn new() -> MockAgent {
        let agent_info = Ok(AgentInfo::new(AgentVersion::new("dcd", "1.2.3", "tainted")));
        let datastore_info = Ok(DatastoreInfo::new(
            "id",
            "DB",
            "mock",
            "1.2.3",
            Some("display".into()),
        ));
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
        self.agent_info
            .clone()
            .map_err(|error| ErrorKind::FreeForm(error).into())
    }

    fn datastore_info(&self, _: &mut Span) -> Result<DatastoreInfo> {
        self.datastore_info
            .clone()
            .map_err(|error| ErrorKind::FreeForm(error).into())
    }

    fn shards(&self, _: &mut Span) -> Result<Shards> {
        self.shards
            .clone()
            .map_err(|error| ErrorKind::FreeForm(error).into())
    }
}

impl Default for MockAgent {
    fn default() -> MockAgent {
        MockAgent::new()
    }
}
