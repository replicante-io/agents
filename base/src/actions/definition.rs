use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Abstraction of any action the agent can perform.
///
/// # Action IDs
/// Action IDs must be scoped to limit the chance of clashes.
/// Scoping is done using the `<SCOPE>.<ACTION>` format.
/// An action ID can have as many `.`s in it as desired, making Java-like reverse DNS
/// scopes an option that greatly reduces the chances of clashes.
///
/// The only constraint on Action IDs is that some scopes are reserved to replicante use itself.
/// This allows the base agent frameworks to define some standard actions across all agents
/// without clashing with custom or database specific actions.
pub trait Action: Send + Sync + 'static {
    /// Action metadata and attributes.
    fn describe(&self) -> ActionDescriptor;
}

/// Container for an action's metadata and other attributes.
///
/// This data is the base of the actions system.
/// Instead of hardcoded knowledge about what actions do,
/// both system and users rely on metadata to call actions.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionDescriptor {
    pub id: String,
    pub description: String,
}
