use std::collections::HashSet;

use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use failure::ResultExt;
use serde_json::json;

use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_util_actixweb::request_span;
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
pub async fn info(id: web::Path<String>, request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let id = id.into_inner();
    let mut exts = request.extensions_mut();
    let span = request_span(&mut exts);
    let info = context
        .store
        .with_transaction(|tx| {
            let action = tx.action().get(&id, span.context().clone())?;
            let action = match action {
                None => return Ok(None),
                Some(action) => action.into(),
            };
            let iter = tx.action().history(&id, span.context().clone())?;
            let mut history = Vec::new();
            for item in iter {
                history.push(item?);
            }
            let info = ActionInfoResponse { action, history };
            Ok(Some(info))
        })
        .map_err(|error| fail_span(error, span))?;
    match info {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(info) => Ok(HttpResponse::Ok().json(info)),
    }
}

/// Attempt to schedule an action.
pub async fn schedule(
    kind: web::Path<String>,
    params: web::Json<ActionScheduleRequest>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let mut exts = request.extensions_mut();
    let span = request_span(&mut exts);
    let kind = kind.into_inner();
    let action = ACTIONS::get(&kind)
        .ok_or_else(|| ErrorKind::ActionNotAvailable(kind.clone()))
        .map_err(Error::from)
        .map_err(|error| fail_span(error, &mut *span))?;

    let params = params.into_inner();
    let args = params.args;
    let created_ts = params.created_ts;
    let action_id = params.action_id;
    action
        .validate_args(&args)
        .map_err(|error| fail_span(error, &mut *span))?;

    let requester = params.requester.unwrap_or(ActionRequester::AgentApi);
    let mut record = ActionRecord::new(kind, action_id, created_ts, args, requester);
    for (name, value) in request.headers() {
        let name = name.as_str();
        if HTTP_HEADER_IGNORE.contains(name) {
            continue;
        }
        let name = name.to_string();
        let value = value
            .to_str()
            .with_context(|_| ErrorKind::ActionEncode)
            .map_err(Error::from)
            .map_err(|error| fail_span(error, &mut *span))?
            .to_string();
        record.headers.insert(name, value);
    }
    record
        .trace_set(span.context(), &context.tracer)
        .map_err(Error::from)
        .map_err(|error| fail_span(error, &mut *span))?;
    let id = record.id;
    context
        .store
        .with_transaction(|tx| tx.action().insert(record, span.context().clone()))
        .map_err(|error| fail_span(error, span))?;
    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}
