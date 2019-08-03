use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use serde_json::Value;

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
    let action = context.store.with_transaction(|tx| tx.action().get(&id))?;
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
    let kind = kind.into_inner();
    let action = ACTIONS::get(&kind)
        .ok_or_else(|| ErrorKind::ActionNotAvailable(kind.clone()))
        .map_err(Error::from)?;
    action.validate_args(&args)?;
    let record = ActionRecord::new(kind, args.into_inner(), ActionRequester::Api);
    let id = record.id;
    context
        .store
        .with_transaction(|tx| tx.persist().action(record))?;
    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}
