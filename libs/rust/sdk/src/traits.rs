use std::sync::Arc;

use opentracingrust::Span;

use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;

use crate::actions::Action;
use crate::Result;

/// Trait to share common agent code and features.
///
/// Agents should be implemented as structs that implement `BaseAgent`.
pub trait Agent: Send + Sync {
    /// Fetches the agent version information.
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo>;

    /// Fetches the datastore information.
    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo>;

    /// Name of the datastore service for service-related actions.
    fn service_name(&self) -> String;

    /// Fetches all shards and details on the managed datastore node.
    fn shards(&self, span: &mut Span) -> Result<Shards>;

    /// Factory for an optional `replicante.store.stop` action.
    ///
    /// Such action, if returned MUST implement a datastore specific graceful shutdown.
    /// This action is not expected to operate on the process itself, although it will
    /// likely cause it to exit.
    ///
    /// If a datastore does not have any such action, let the default implementation
    /// return `None` for you to indicate this.
    ///
    /// For example, MongoDB `db.shutdownServer` is a good candidate for this action.
    fn graceful_stop_action(&self) -> Option<Arc<dyn Action>> {
        None
    }
}
