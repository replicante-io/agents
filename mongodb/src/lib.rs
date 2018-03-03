#[macro_use(bson, doc)]
extern crate bson;
extern crate config;
extern crate mongodb;
extern crate opentracingrust;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate replicante_agent;

use bson::Bson;
use bson::ordered::OrderedDocument;

use mongodb::Client;
use mongodb::CommandType;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::Tracer;
use opentracingrust::utils::FailSpan;

use replicante_agent::Agent;
use replicante_agent::AgentError;
use replicante_agent::AgentResult;

use replicante_agent::models::AgentVersion;
use replicante_agent::models::DatastoreVersion;
use replicante_agent::models::Shard;
use replicante_agent::models::ShardRole;

pub mod settings;
mod error;
mod rs_status;

use self::settings::MongoDBSettings;


/// Agent dealing with MongoDB 3.2+ Replica Sets.
pub struct MongoDBAgent {
    // The client needs to reference mongo settings inside the agent.
    // To implement this, the client is stored in an option that is
    // filled just after the agent is created while in the factory.
    client: Option<Client>,
    settings: MongoDBSettings,
    tracer: Tracer,
}

impl MongoDBAgent {
    pub fn new(settings: MongoDBSettings, tracer: Tracer) -> AgentResult<MongoDBAgent> {
        let mut agent = MongoDBAgent {
            client: None,
            tracer,
            settings: settings,
        };
        agent.init_client()?;
        Ok(agent)
    }
}

impl MongoDBAgent {
    /// Initialises the MongoDB client instance for the agent.
    fn init_client(&mut self) -> AgentResult<()> {
        let host = &self.settings.host;
        let port = self.settings.port as u16;
        let client = Client::connect(host, port)
            .map_err(self::error::to_agent)?;
        self.client = Some(client);
        Ok(())
    }

    /// Extract the client from the wrapping `Option`.
    fn client(&self) -> &Client {
        self.client.as_ref().unwrap()
    }

    /// Executes the buildInfo command against the DB.
    fn build_info(&self, parent: &mut Span) -> AgentResult<OrderedDocument> {
        let mut span = self.tracer().span("buildInfo").auto_finish();
        span.child_of(parent.context().clone());
        let mongo = self.client();
        span.log(Log::new().log("span.kind", "client-send"));
        let info = mongo.db("test").command(
            doc! {"buildInfo" => 1},
            CommandType::BuildInfo,
            None
        ).fail_span(&mut span).map_err(self::error::to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(info)
    }

    /// Executes the replSetGetStatus command against the DB.
    fn repl_set_get_status(&self, parent: &mut Span) -> AgentResult<OrderedDocument> {
        let mut span = self.tracer().span("replSetGetStatus").auto_finish();
        span.child_of(parent.context().clone());
        let mongo = self.client();
        span.log(Log::new().log("span.kind", "client-send"));
        let status = mongo.db("admin").command(
            doc! {"replSetGetStatus" => 1},
            CommandType::IsMaster,
            None
        ).fail_span(&mut span).map_err(self::error::to_agent)?;
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(status)
    }
}

impl Agent for MongoDBAgent {
    fn agent_version(&self, _: &mut Span) -> AgentResult<AgentVersion> {
        Ok(AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
        ))
    }

    fn datastore_version(&self, span: &mut Span) -> AgentResult<DatastoreVersion> {
        let info = self.build_info(span)?;
        let version = info.get("version").ok_or(AgentError::ModelViolation(
            String::from("Unable to determine MongoDB version")
        ))?;
        if let &Bson::String(ref version) = version {
            Ok(DatastoreVersion::new("MongoDB", version))
        } else {
            Err(AgentError::ModelViolation(String::from(
                "Unexpeted version type (should be String)"
            )))
        }
    }

    fn tracer(&self) -> &Tracer {
        &self.tracer
    }

    fn shards(&self, span: &mut Span) -> AgentResult<Vec<Shard>> {
        let status = self.repl_set_get_status(span)?;
        let name = rs_status::name(&status)?;
        let role = rs_status::role(&status)?;
        let last_op = rs_status::last_op(&status)?;
        let lag = match role {
            ShardRole::Primary => 0,
            _ => rs_status::lag(&status, last_op)?
        };
        Ok(vec![Shard::new(&name, role, lag, last_op)])
    }
}
