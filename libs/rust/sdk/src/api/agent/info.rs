use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use opentracingrust::Log;

use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_tracing::fail_span;

use crate::Agent;
use crate::AgentContext;

/// API interface to Agent::agent_info
pub fn agent(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::new(logger, tracer);
    web::resource("/agent")
        .wrap(tracer)
        .route(web::get().to(agent_respoder))
}

async fn agent_respoder(
    agent: web::Data<Arc<dyn Agent>>,
    mut request: HttpRequest,
) -> Result<impl Responder> {
    with_request_span(&mut request, |span| {
        let span = span.expect("unable to find tracing span for request");
        span.log(Log::new().log("span.kind", "server-receive"));
        let info = agent
            .agent_info(span)
            .map_err(|error| fail_span(error, &mut *span))?;
        let response = HttpResponse::Ok().json(info);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    })
}

/// API interface to Agent::datastore_info
pub fn datastore(context: &AgentContext) -> impl HttpServiceFactory {
    let cluster_display_name_override = context.config.cluster_display_name_override.clone();
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::new(logger, tracer);
    web::resource("/datastore")
        .data(cluster_display_name_override)
        .wrap(tracer)
        .route(web::get().to(datastore_responder))
}

async fn datastore_responder(
    agent: web::Data<Arc<dyn Agent>>,
    cluster_display_name_override: web::Data<Option<String>>,
    mut request: HttpRequest,
) -> Result<impl Responder> {
    with_request_span(&mut request, |span| {
        let span = span.expect("unable to find tracing span for request");
        span.log(Log::new().log("span.kind", "server-receive"));
        let mut info = agent
            .datastore_info(span)
            .map_err(|error| fail_span(error, &mut *span))?;

        // Inject the cluster_display_name override if configured.
        info.cluster_display_name = cluster_display_name_override
            .as_ref()
            .as_ref()
            .cloned()
            .or(info.cluster_display_name);

        let response = HttpResponse::Ok().json(info);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    })
}
