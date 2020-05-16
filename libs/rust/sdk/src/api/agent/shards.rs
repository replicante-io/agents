use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use opentracingrust::Log;

use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_tracing::fail_span;

use crate::Agent;
use crate::AgentContext;
use crate::Result;

/// API interface to Agent::shards
pub fn shards(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::new(logger, tracer);
    web::resource("/shards")
        .wrap(tracer)
        .route(web::get().to(shards_responder))
}

async fn shards_responder(
    agent: web::Data<Arc<dyn Agent>>,
    mut request: HttpRequest,
) -> Result<impl Responder> {
    with_request_span(&mut request, |span| {
        let span = span.expect("unable to find tracing span for request");
        span.log(Log::new().log("span.kind", "server-receive"));
        let shards = agent
            .shards(span)
            .map_err(|error| fail_span(error, &mut *span))?;
        let response = HttpResponse::Ok().json(shards);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    })
}
