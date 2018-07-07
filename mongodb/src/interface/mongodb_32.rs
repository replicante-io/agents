use bson::Bson;
use bson::ordered::OrderedDocument;

use mongodb::Client;
use mongodb::CommandType;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::utils::FailSpan;

use replicante_agent::AgentContext;
use replicante_agent::Error;
use replicante_agent::Result;

use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shard;
use replicante_agent_models::Shards;
use replicante_agent_models::ShardRole;

use super::super::errors;
use super::super::rs_status;
use super::super::metrics::MONGO_COMMAND_COUNTS;

use super::MongoDBInterface;


/// Section of the output from the buildInfo command that we care about.
#[derive(Deserialize)]
pub struct BuildInfo {
    pub version: String,
}


pub struct MongoClient {
    context: AgentContext,
}

impl MongoClient {
    pub fn new(context: AgentContext) -> MongoClient {
        MongoClient {
            context,
        }
    }

    /// Executes the buildInfo command against the DB.
    fn build_info(&self, parent: &mut Span, client: &Client) -> Result<OrderedDocument> {
        let mut span = self.context.tracer.span("buildInfo").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        MONGO_COMMAND_COUNTS.with_label_values(&["buildInfo"]).inc();
        let info = client.db("test").command(
            doc! {"buildInfo" => 1},
            CommandType::BuildInfo,
            None
        ).fail_span(&mut span).map_err(errors::to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(info)
    }

    /// Executes the replSetGetStatus command against the DB.
    fn repl_set_get_status(&self, parent: &mut Span, client: &Client) -> Result<OrderedDocument> {
        let mut span = self.context.tracer.span("replSetGetStatus").auto_finish();
        span.child_of(parent.context().clone());
        span.log(Log::new().log("span.kind", "client-send"));
        MONGO_COMMAND_COUNTS.with_label_values(&["replSetGetStatus"]).inc();
        let status = client.db("admin").command(
            doc! {"replSetGetStatus" => 1},
            CommandType::IsMaster,
            None
        ).fail_span(&mut span).map_err(errors::to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(status)
    }
}

impl MongoDBInterface for MongoClient {
    fn datastore_info(&self, span: &mut Span, client: &Client) -> Result<DatastoreInfo> {
        let info = self.build_info(span, client)?;
        let version = info.get("version").ok_or_else(
            || Error::from("Unable to determine MongoDB version")
        )?;
        if let Bson::String(ref version) = *version {
            let status = self.repl_set_get_status(span, client)?;
            let cluster = rs_status::name(&status)?;
            let node_name = rs_status::node_name(&status)?;
            Ok(DatastoreInfo::new(cluster, "MongoDB", node_name, version.clone()))
        } else {
            Err("Unexpeted version type (should be String)".into())
        }
    }

    fn shards(&self, span: &mut Span, client: &Client) -> Result<Shards> {
        let status = self.repl_set_get_status(span, client)?;
        let name = rs_status::name(&status)?;
        let role = rs_status::role(&status)?;
        let last_op = rs_status::last_op(&status)?;
        let lag = match role {
            ShardRole::Primary => Some(0),
            _ => match rs_status::lag(&status, last_op) {
                Ok(lag) => Some(lag),
                Err(err) => {
                    let error = format!("{:?}", err);
                    error!(self.context.logger, "Failed to compute lag"; "error" => error);
                    span.tag("lag.error", format!("Failed lag detection: {:?}", err));
                    None
                }
            }
        };
        let shards = vec![Shard::new(name, role, lag, last_op)];
        Ok(Shards::new(shards))
    }
}
