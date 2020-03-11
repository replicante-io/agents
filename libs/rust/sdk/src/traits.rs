use std::sync::Arc;

use opentracingrust::Span;

use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;

use crate::actions::Action;
use crate::actions::ActionHook;
use crate::Result;

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

    /// Factory for store-specific well-known actions.
    ///
    /// These actions are part of the SDK reserved scope so they have well defined expectations
    /// but thier implementation is delegated to specific agents.
    ///
    /// This allows standard actions with store-specific implementation that can be used to
    /// build reusable, standard, cross-store logic.
    fn action_hooks(&self) -> Vec<(ActionHook, Arc<dyn Action>)> {
        Vec::new()
    }
}
