use std::sync::Arc;

use bson::doc;
use bson::Bson;
use failure::ResultExt;
use mongodb::Client;
use opentracingrust::utils::FailSpan;
use opentracingrust::Log;
use opentracingrust::Span;
use slog::error;

use replicante_agent::actions::Action;
use replicante_agent::actions::ActionHook;
use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;
use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::CommitOffset;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shard;
use replicante_models_agent::info::ShardRole;
use replicante_models_agent::info::Shards;
use replicante_util_failure::failure_info;

use crate::actions::GracefulStop;
use crate::error::ErrorKind;
use crate::metrics::MONGODB_OPS_COUNT;
use crate::metrics::MONGODB_OPS_DURATION;
use crate::metrics::MONGODB_OP_ERRORS_COUNT;
use crate::version::common::AGENT_VERSION;

use super::BuildInfo;
use super::ReplSetStatus;

/// MongoDB 3.0 replica set agent.
pub struct ReplicaSet {
    client: Client,
    context: AgentContext,
}

impl ReplicaSet {
    pub fn new(client: Client, context: AgentContext) -> ReplicaSet {
        ReplicaSet { client, context }
    }

    /// Executes the buildInfo command against the DB.
    fn build_info(&self, parent: &mut Span) -> Result<BuildInfo> {
        let mut span = self.context.tracer.span("buildInfo").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        MONGODB_OPS_COUNT.with_label_values(&["buildInfo"]).inc();
        let timer = MONGODB_OPS_DURATION
            .with_label_values(&["buildInfo"])
            .start_timer();
        let info = self
            .client
            .database("test")
            .run_command(doc! {"buildInfo": 1}, None)
            .fail_span(&mut span)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT
                    .with_label_values(&["buildInfo"])
                    .inc();
                error
            })
            .with_context(|_| ErrorKind::StoreOpFailed("buildInfo"))?;
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        let info = bson::from_bson(Bson::Document(info))
            .with_context(|_| ErrorKind::BsonDecode("buildInfo"))?;
        Ok(info)
    }

    /// Executes the replSetGetStatus command against the DB.
    fn repl_set_get_status(&self, parent: &mut Span) -> Result<ReplSetStatus> {
        let mut span = self.context.tracer.span("replSetGetStatus").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        MONGODB_OPS_COUNT
            .with_label_values(&["replSetGetStatus"])
            .inc();
        let timer = MONGODB_OPS_DURATION
            .with_label_values(&["replSetGetStatus"])
            .start_timer();
        let status = self
            .client
            .database("admin")
            .run_command(doc! {"replSetGetStatus" => 1}, None)
            .fail_span(&mut span)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT
                    .with_label_values(&["replSetGetStatus"])
                    .inc();
                error
            })
            .with_context(|_| ErrorKind::StoreOpFailed("replSetGetStatus"))?;
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        let status = bson::from_bson(Bson::Document(status))
            .with_context(|_| ErrorKind::BsonDecode("replSetGetStatus"))?;
        Ok(status)
    }
}

impl Agent for ReplicaSet {
    fn action_hooks(&self) -> Vec<(ActionHook, Arc<dyn Action>)> {
        vec![(
            ActionHook::StoreGracefulStop,
            Arc::new(GracefulStop::new(self.client.clone())),
        )]
    }

    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        span.log(Log::new().log("span.kind", "server-receive"));
        let info = AgentInfo::new(AGENT_VERSION.clone());
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let info = self.build_info(span)?;
        let status = self.repl_set_get_status(span)?;
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

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        let status = self.repl_set_get_status(span)?;
        let last_op = status.last_op()?;
        let role = status.role()?;
        let lag = match role {
            ShardRole::Primary => None,
            _ => match status.primary_optime() {
                Ok(head) => Some(CommitOffset::seconds(head - last_op)),
                Err(error) => {
                    error!(self.context.logger, "Failed to compute lag"; failure_info(&error));
                    span.tag("lag.error", format!("Failed lag computation: {:?}", error));
                    None
                }
            },
        };
        let name = status.set;
        let shards = vec![Shard::new(
            name,
            role,
            Some(CommitOffset::seconds(last_op)),
            lag,
        )];
        Ok(Shards::new(shards))
    }
}
