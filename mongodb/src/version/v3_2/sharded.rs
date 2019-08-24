use mongodb::Client;
use opentracingrust::Span;

use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_models_agent::AgentInfo;
use replicante_models_agent::DatastoreInfo;
use replicante_models_agent::Shards;

use super::super::Sharding;
use super::common::CommonLogic;

/// MongoDB 3.2+ sharded agent.
pub struct Sharded {
    cluster_name: String,
    common: CommonLogic,
    is_mongos: bool,
    mongos_node_name: Option<String>,
}

impl Sharded {
    pub fn new(sharding: Sharding, client: Client, context: AgentContext) -> Sharded {
        let common = CommonLogic::new(client, context);
        let is_mongos = sharding.mongos_node_name.is_some();
        Sharded {
            cluster_name: sharding.cluster_name,
            common,
            is_mongos,
            mongos_node_name: sharding.mongos_node_name,
        }
    }
}

impl Agent for Sharded {
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        self.common.agent_info(span)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let info = self.common.build_info(span)?;
        let cluster = self.cluster_name.clone();
        if self.is_mongos {
            let node_name = self.mongos_node_name.as_ref().unwrap().clone();
            Ok(DatastoreInfo::new(
                cluster,
                "MongoDB",
                node_name,
                info.version,
                None,
            ))
        } else {
            let status = self.common.repl_set_get_status(span)?;
            let node_name = status.node_name()?;
            Ok(DatastoreInfo::new(
                cluster,
                "MongoDB",
                node_name,
                info.version,
                None,
            ))
        }
    }

    fn service_name(&self) -> String {
        "mongod".into()
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        if self.is_mongos {
            Ok(Shards::new(Vec::new()))
        } else {
            self.common.shards(span)
        }
    }
}
