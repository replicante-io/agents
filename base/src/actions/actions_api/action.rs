use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use serde_json::Value;

use replicante_util_actixweb::request_span;
use replicante_util_tracing::fail_span;

use crate::actions::ActionRecord;
use crate::actions::ActionRequester;
use crate::actions::ACTIONS;
use crate::AgentContext;
use crate::Error;
use crate::ErrorKind;

/// Action information returned by the API.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionInfo {
    action: ActionRecord,
    // TODO: add action history.
}

/// Fetch an action details.
pub fn info(id: web::Path<String>, request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let id = id.into_inner();
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    let action = context
        .store
        .with_transaction(|tx| tx.action().get(&id, span.context().clone()))
        .map_err(|error| fail_span(error, &mut span))?;
    let action = match action {
        None => return Ok(HttpResponse::NotFound().finish()),
        Some(action) => action,
    };
    let info = ActionInfo { action };
    Ok(HttpResponse::Ok().json(info))
}

/// Attempt to schedule an action.
pub fn schedule(
    kind: web::Path<String>,
    args: web::Json<Value>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    let kind = kind.into_inner();
    let action = ACTIONS::get(&kind)
        .ok_or_else(|| ErrorKind::ActionNotAvailable(kind.clone()))
        .map_err(Error::from)
        .map_err(|error| fail_span(error, &mut span))?;
    action.validate_args(&args)?;
    let record = ActionRecord::new(kind, args.into_inner(), ActionRequester::Api);
    let id = record.id;
    context
        .store
        .with_transaction(|tx| tx.action().insert(record, span.context().clone()))
        .map_err(|error| fail_span(error, &mut span))?;
    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}