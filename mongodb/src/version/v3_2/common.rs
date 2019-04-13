use bson;
use bson::Bson;
use failure::ResultExt;

use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::CommandType;
use mongodb::ThreadedClient;

use opentracingrust::utils::FailSpan;
use opentracingrust::Log;
use opentracingrust::Span;

use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::CommitOffset;
use replicante_agent_models::Shard;
use replicante_agent_models::ShardRole;
use replicante_agent_models::Shards;
use replicante_util_failure::failure_info;

use super::super::super::error::ErrorKind;
use super::super::super::metrics::MONGODB_OPS_COUNT;
use super::super::super::metrics::MONGODB_OPS_DURATION;
use super::super::super::metrics::MONGODB_OP_ERRORS_COUNT;
use super::super::common::AGENT_VERSION;

use super::BuildInfo;
use super::ReplSetStatus;

/// MongoDB 3.2+ logic common to both RS and Shareded modes.
pub struct CommonLogic {
    client: Client,
    context: AgentContext,
}

impl CommonLogic {
    pub fn new(client: Client, context: AgentContext) -> CommonLogic {
        CommonLogic { client, context }
    }

    /// Returns agent information.
    pub fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        let info = AgentInfo::new(AGENT_VERSION.clone());
        Ok(info)
    }

    /// Executes the buildInfo command against the DB.
    pub fn build_info(&self, parent: &mut Span) -> Result<BuildInfo> {
        let mut span = self.context.tracer.span("buildInfo").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        MONGODB_OPS_COUNT.with_label_values(&["buildInfo"]).inc();
        let timer = MONGODB_OPS_DURATION
            .with_label_values(&["buildInfo"])
            .start_timer();
        let info = self
            .client
            .db("test")
            .command(doc! {"buildInfo" => 1}, CommandType::BuildInfo, None)
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
    pub fn repl_set_get_status(&self, parent: &mut Span) -> Result<ReplSetStatus> {
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
            .db("admin")
            .command(doc! {"replSetGetStatus" => 1}, CommandType::IsMaster, None)
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

    /// Returns shard information from a MongoD instance.
    pub fn shards(&self, span: &mut Span) -> Result<Shards> {
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
