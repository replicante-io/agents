use std::sync::Arc;

use actix_web::web;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

pub mod info;
pub mod shards;

use self::info::AgentInfo;
use self::info::DatastoreInfo;
use self::shards::Shards;

use super::APIRoot;
use crate::Agent;
use crate::AgentContext;

/// Configure all agent endpoints.
pub fn configure_app(
    flags: &APIFlags,
    app: &mut web::ServiceConfig,
    agent: Arc<dyn Agent>,
    context: &AgentContext,
) {
    let tracer = Arc::clone(&context.tracer);
    APIRoot::UnstableAPI.and_then(flags, |root| {
        let agent_for_agent = Arc::clone(&agent);
        let agent_for_datastore = Arc::clone(&agent);
        let agent_for_shards = agent;
        let cluster_display_name_override = context.config.cluster_display_name_override.clone();
        app.service(
            root.resource("/info/agent")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(move || AgentInfo::new(Arc::clone(&agent_for_agent)))),
        );
        app.service(
            root.resource("/info/datastore")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(move || {
                    DatastoreInfo::new(
                        Arc::clone(&agent_for_datastore),
                        cluster_display_name_override.clone(),
                    )
                })),
        );
        app.service(
            root.resource("/shards")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(move || Shards::new(Arc::clone(&agent_for_shards)))),
        );
    });
}
