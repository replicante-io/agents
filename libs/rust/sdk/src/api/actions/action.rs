use std::collections::HashSet;
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use failure::ResultExt;
use serde_json::json;

use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_tracing::fail_span;

use crate::actions::ActionRecord;
use crate::actions::ActionRequester;
use crate::actions::ACTIONS;
use crate::AgentContext;
use crate::Error;
use crate::ErrorKind;

lazy_static::lazy_static! {
    /// Set of HTTP headers to exclude when collecting action headers.
    static ref HTTP_HEADER_IGNORE: HashSet<String> = {
        let mut headers = HashSet::new();
        headers.insert("accept".into());
        headers.insert("accept-encoding".into());
        headers.insert("content-length".into());
        headers.insert("content-type".into());
        headers.insert("host".into());
        headers.insert("user-agent".into());
        headers
    };
}

/// Fetch an action details.
pub fn info(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::with_name(logger, tracer, "/actions/info/{id}");
    web::resource("/info/{id}")
        .wrap(tracer)
        .route(web::get().to(info_responder))
}

async fn info_responder(
    context: web::Data<AgentContext>,
    id: web::Path<String>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let mut request = request;
    let id = id.into_inner();
    let info = with_request_span(&mut request, |span| {
        let span_context = span.as_ref().map(|span| span.context().clone());
        context
            .store
            .with_transaction(|tx| {
                let action = tx.action().get(&id, span_context.clone())?;
                let action = match action {
                    None => return Ok(None),
                    Some(action) => action.into(),
                };
                let iter = tx.action().history(&id, span_context)?;
                let mut history = Vec::new();
                for item in iter {
                    history.push(item?);
                }
                let info = ActionInfoResponse { action, history };
                Ok(Some(info))
            })
            .map_err(|error| fail_span(error, span))
    })?;
    match info {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(info) => Ok(HttpResponse::Ok().json(info)),
    }
}

/// Attempt to schedule an action.
pub fn schedule(context: &AgentContext) -> impl HttpServiceFactory {
    let logger = context.logger.clone();
    let tracer = Arc::clone(&context.tracer);
    let tracer = TracingMiddleware::with_name(logger, tracer, "/actions/schedule/{kind}");
    web::resource("/schedule/{kind:.*}")
        .wrap(tracer)
        .route(web::post().to(schedule_responder))
}

async fn schedule_responder(
    context: web::Data<AgentContext>,
    kind: web::Path<String>,
    params: web::Json<ActionScheduleRequest>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let mut request = request;
    let kind = kind.into_inner();
    let action = with_request_span(&mut request, |span| {
        ACTIONS::get(&kind)
            .ok_or_else(|| ErrorKind::ActionNotAvailable(kind.clone()))
            .map_err(Error::from)
            .map_err(|error| fail_span(error, span))
    })?;

    let params = params.into_inner();
    let args = params.args;
    let created_ts = params.created_ts;
    let action_id = params.action_id;
    with_request_span(&mut request, |span| {
        action
            .validate_args(&args)
            .map_err(|error| fail_span(error, span))
    })?;

    let requester = params.requester.unwrap_or(ActionRequester::AgentApi);
    let mut record = ActionRecord::new(kind, action_id, created_ts, args, requester);
    let headers = request.headers().clone();
    for (name, value) in headers.into_iter() {
        let name = name.as_str();
        if HTTP_HEADER_IGNORE.contains(name) {
            continue;
        }
        let name = name.to_string();
        let value = with_request_span(&mut request, |span| -> Result<_> {
            let value = value
                .to_str()
                .with_context(|_| ErrorKind::ActionEncode)
                .map_err(Error::from)
                .map_err(|error| fail_span(error, span))?
                .to_string();
            Ok(value)
        })?;
        record.headers.insert(name, value);
    }
    with_request_span(&mut request, |span| -> Result<_> {
        let span_context = span.as_ref().map(|span| span.context().clone());
        if let Some(span_context) = span_context.as_ref() {
            record
                .trace_set(span_context, &context.tracer)
                .map_err(Error::from)
                .map_err(|error| fail_span(error, span))?;
        }
        Ok(())
    })?;
    let id = record.id;
    with_request_span(&mut request, |span| -> Result<_> {
        let span_context = span.as_ref().map(|span| span.context().clone());
        context
            .store
            .with_transaction(|tx| tx.action().insert(record, span_context))
            .map_err(|error| fail_span(error, span))?;
        Ok(())
    })?;
    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}
