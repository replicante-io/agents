use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use serde_json::Value;

use crate::actions::ACTIONS;
use crate::Error;
use crate::ErrorKind;

/// Attempt to schedule an action.
pub fn responder(kind: web::Path<String>, args: web::Json<Value>) -> Result<impl Responder> {
    let action = ACTIONS::get(&kind)
        .ok_or_else(|| ErrorKind::ActionNotAvailable(kind.into_inner()))
        .map_err(Error::from)?;
    action.validate_args(&args)?;
    //println!("{:?} => {:?}", kind, args);
    Ok(HttpResponse::Ok())
}
