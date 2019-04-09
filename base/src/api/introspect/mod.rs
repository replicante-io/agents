use replicante_util_iron::MetricsHandler;
use replicante_util_iron::Router;

use super::APIRoot;
use super::AgentContext;

mod threads;

/// Mount all introspection API endpoints onto the router.
pub fn mount(context: &AgentContext, router: &mut Router) {
    let registry = context.metrics.clone();
    let mut root = router.for_root(&APIRoot::UnstableIntrospect);
    root.get("/metrics", MetricsHandler::new(registry), "/metrics");
    root.get("/threads", threads::handler, "/threads");
}
