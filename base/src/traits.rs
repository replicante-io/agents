use opentracingrust::Span;

use replicante_models_agent::AgentInfo;
use replicante_models_agent::DatastoreInfo;
use replicante_models_agent::Shards;

use super::Result;

/// Trait to share common agent code and features.
///
/// Agents should be implemented as structs that implement `BaseAgent`.
pub trait Agent: Send + Sync {
    /// Fetches the agent version information.
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo>;

    /// Fetches the datastore information.
    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo>;

    /// Fetches all shards and details on the managed datastore node.
    fn shards(&self, span: &mut Span) -> Result<Shards>;
}
