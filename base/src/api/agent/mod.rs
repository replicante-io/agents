use std::sync::Arc;

use replicante_util_iron::Router;

pub mod info;
pub mod shards;

use self::info::AgentInfo;
use self::info::DatastoreInfo;
use self::shards::Shards;

use super::APIRoot;
use super::Agent;
use super::AgentContext;

/// Mount all agent API endpoints onto the router.
pub fn mount(agent: Arc<dyn Agent>, context: AgentContext, router: &mut Router) {
    let agent_info = AgentInfo::make(Arc::clone(&agent));
    let datastore_info = DatastoreInfo::make(Arc::clone(&agent), context);
    let shards = Shards::make(agent);
    let mut root = router.for_root(&APIRoot::UnstableAPI);
    root.get("/info/agent", agent_info, "/info/agent");
    root.get("/info/datastore", datastore_info, "/info/datastore");
    root.get("/shards", shards, "/shards");
}
