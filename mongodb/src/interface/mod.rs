use mongodb::Client;
use opentracingrust::Span;

use replicante_agent::Result;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;


pub mod mongodb_32;


/// Version dependent MongoDB agents.
pub trait MongoDBInterface : Send + Sync {
    /// Access datastore info for this version.
    fn datastore_info(&self, span: &mut Span, client: &Client) -> Result<DatastoreInfo>;

    /// Access shards info for this version.
    fn shards(&self, span: &mut Span, client: &Client) -> Result<Shards>;
}
