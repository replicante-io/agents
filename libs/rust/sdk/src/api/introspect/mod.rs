use actix_web::web;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::MetricsExporter;
use replicante_util_actixweb::RootDescriptor;

use super::APIRoot;
use crate::AgentContext;

mod threads;

/// Configure all introspection endpoints.
pub fn configure_app(context: &AgentContext, flags: &APIFlags, app: &mut web::ServiceConfig) {
    APIRoot::UnstableIntrospect.and_then(flags, |root| {
        let registry = context.metrics.clone();
        app.service(
            root.resource("/metrics")
                .route(web::get().to(MetricsExporter::factory(registry))),
        );
        app.service(
            root.resource("/threads")
                .route(web::get().to(threads::handler)),
        );
    });
}
