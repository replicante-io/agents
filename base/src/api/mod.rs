use std::collections::HashMap;
use std::sync::Arc;

use iron::Chain;
use iron_json_response::JsonResponseMiddleware;

use replicante_util_iron::MetricsMiddleware;
use replicante_util_iron::RequestLogger;
use replicante_util_iron::RootDescriptor;
use replicante_util_iron::Router;

mod agent;
mod index;
mod introspect;
mod metrics;

pub use self::metrics::register_metrics;

use super::Agent;
use super::AgentContext;

/// Mount all API endpoints into an Iron Chain.
pub fn mount(agent: Arc<Agent>, context: AgentContext) -> Chain {
    let logger = context.logger.clone();
    let mut router = Router::new(context.config.api.trees.clone().into());

    // Create the index root for each API root.
    let roots = vec![APIRoot::UnstableAPI];
    for root in roots.into_iter() {
        let mut root = router.for_root(root);
        root.get("/", index::index, "index");
    }

    // Mount endpooints.
    self::introspect::mount(&context, &mut router);
    self::agent::mount(agent, context, &mut router);

    // Build and return the Iron Chain.
    let mut chain = router.build();
    let metrics_middlewere = MetricsMiddleware::new(
        self::metrics::MIDDLEWARE.0.clone(),
        self::metrics::MIDDLEWARE.1.clone(),
        self::metrics::MIDDLEWARE.2.clone(),
        logger.clone(),
    );
    chain.link_after(JsonResponseMiddleware::new());
    chain.link_after(RequestLogger::new(logger));
    chain.link(metrics_middlewere.into_middleware());
    chain
}

/// Enumerates all possible API roots.
///
/// All endpoints must fall under one of these roots and are subject to all restrictions
/// of that specific root.
/// The main restriction is that versioned APIs are subject to semver guarantees.
pub enum APIRoot {
    /// API root for all endpoints that are not yet stable.
    ///
    /// Endpoints in this root are NOT subject to ANY compatibility guarantees!
    UnstableAPI,

    /// Instrospection APIs not yet stable.
    UnstableIntrospect,
}

impl RootDescriptor for APIRoot {
    fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
        match self {
            APIRoot::UnstableAPI => match flags.get("unstable") {
                Some(flag) => *flag,
                None => true,
            },
            APIRoot::UnstableIntrospect => match flags.get("unstable") {
                Some(flag) if !flag => false,
                _ => match flags.get("introspect") {
                    Some(flag) => *flag,
                    None => true,
                },
            },
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            APIRoot::UnstableAPI => "/api/unstable",
            APIRoot::UnstableIntrospect => "/api/unstable/introspect",
        }
    }
}
