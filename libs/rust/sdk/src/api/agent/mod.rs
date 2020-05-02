use std::sync::Arc;

use actix_web::web;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

pub mod info;
pub mod shards;

use super::APIRoot;
use crate::AgentContext;

/// Configure all agent endpoints.
pub fn configure_app(flags: &APIFlags, app: &mut web::ServiceConfig, context: &AgentContext) {
    let tracer = Arc::clone(&context.tracer);
    APIRoot::UnstableAPI.and_then(flags, |root| {
        let cluster_display_name_override = context.config.cluster_display_name_override.clone();
        app.service(
            root.resource("/info/agent")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(info::agent)),
        );
        app.service(
            root.resource("/info/datastore")
                .data(cluster_display_name_override)
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(info::datastore)),
        );
        app.service(
            root.resource("/shards")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(shards::shards)),
        );
    });
}
