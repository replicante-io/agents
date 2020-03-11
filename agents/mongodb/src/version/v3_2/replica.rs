use std::sync::Arc;

use mongodb::Client;
use opentracingrust::Span;

use replicante_agent::actions::Action;
use replicante_agent::actions::ActionHook;
use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;
use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;

use super::common::CommonLogic;
use crate::actions::GracefulStop;

/// MongoDB 3.2+ replica set agent.
pub struct ReplicaSet {
    common: CommonLogic,
}

impl ReplicaSet {
    pub fn new(client: Client, context: AgentContext) -> ReplicaSet {
        let common = CommonLogic::new(client, context);
        ReplicaSet { common }
    }
}

impl Agent for ReplicaSet {
    fn action_hooks(&self) -> Vec<(ActionHook, Arc<dyn Action>)> {
        vec![(
            ActionHook::StoreGracefulStop,
            Arc::new(GracefulStop::new(self.common.client())),
        )]
    }

    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        self.common.agent_info(span)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let info = self.common.build_info(span)?;
        let status = self.common.repl_set_get_status(span)?;
        let node_name = status.node_name()?;
        let cluster = status.set;
        Ok(DatastoreInfo::new(
            cluster,
            "MongoDB",
            node_name,
            info.version,
            None,
        ))
    }

    fn service_name(&self) -> String {
        "mongod".into()
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        self.common.shards(span)
    }
}
