use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;

use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_tracing::fail_span;

use crate::AgentContext;

/// List finished actions.
pub fn finished(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::new(logger, tracer);
    web::resource("/finished")
        .wrap(tracer)
        .route(web::get().to(finished_responder))
}

async fn finished_responder(
    context: web::Data<AgentContext>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let mut request = request;
    let actions = with_request_span(&mut request, |span| {
        let span_context = span.as_ref().map(|span| span.context().clone());
        context
            .store
            .with_transaction(|tx| {
                let mut actions = Vec::new();
                let iter = tx.actions().finished(span_context)?;
                for action in iter {
                    actions.push(action?);
                }
                Ok(actions)
            })
            .map_err(|error| fail_span(error, span))
    })?;
    Ok(HttpResponse::Ok().json(actions))
}

/// List running and pending actions.
pub fn queue(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::new(logger, tracer);
    web::resource("/queue")
        .wrap(tracer)
        .route(web::get().to(queue_responder))
}

async fn queue_responder(
    context: web::Data<AgentContext>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let mut request = request;
    let actions = with_request_span(&mut request, |span| {
        let span_context = span.as_ref().map(|span| span.context().clone());
        context
            .store
            .with_transaction(|tx| {
                let mut actions = Vec::new();
                let iter = tx.actions().queue(span_context)?;
                for action in iter {
                    actions.push(action?);
                }
                Ok(actions)
            })
            .map_err(|error| fail_span(error, span))
    })?;
    Ok(HttpResponse::Ok().json(actions))
}
