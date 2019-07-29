use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use serde_json::Value;

use crate::actions::ACTIONS;
use crate::actions::ActionRecord;
use crate::actions::ActionRequester;
use crate::AgentContext;
use crate::Error;
use crate::ErrorKind;

/// Attempt to schedule an action.
pub fn responder(
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
    context.store.with_transaction(|_tx| {
        println!("{:?}", record);
        //tx.persist().action(record)
        Ok(())
    })?;
    Ok(HttpResponse::Ok())
}
