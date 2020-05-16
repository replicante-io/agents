use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_json::json;

use replicante_util_actixweb::RootDescriptor;

use crate::actions::ActionDescriptor;
use crate::actions::ACTIONS;
use crate::api::APIRoot;
use crate::api::AppConfigContext;

mod action;
mod list;

/// Return a list of available agent actions.
#[actix_web::get("/available")]
async fn available() -> impl Responder {
    let mut actions: Vec<ActionDescriptor> =
        ACTIONS::iter().map(|action| action.describe()).collect();
    actions.sort_by_key(|action| action.kind.clone());
    HttpResponse::Ok().json(actions)
}

/// Static 2xx response to confirm the actions API is NOT enabled.
#[actix_web::get("/actions")]
async fn index_disabled() -> impl Responder {
    HttpResponse::Ok().json(json!({"actions": false}))
}

/// Static 2xx response to confirm the actions API is enabled.
#[actix_web::get("/")]
async fn index_enabled() -> impl Responder {
    HttpResponse::Ok().json(json!({"actions": true}))
}

/// Configure the API server with actions API disabled.
pub fn configure_disabled(conf: &mut AppConfigContext) {
    APIRoot::UnstableAPI.and_then(&conf.context.flags, |root| {
        conf.scoped_service(root.prefix(), index_disabled);
    });
}

/// Configure the API server with actions API enabled.
pub fn configure_enabled(conf: &mut AppConfigContext) {
    APIRoot::UnstableAPI.and_then(&conf.context.flags, |root| {
        let finished = self::list::finished(&conf.context.agent);
        let info = self::action::info(&conf.context.agent);
        let queue = self::list::queue(&conf.context.agent);
        let schedule = self::action::schedule(&conf.context.agent);
        let scope = web::scope("/actions")
            .service(index_enabled)
            .service(available)
            .service(finished)
            .service(queue)
            .service(info)
            .service(schedule);
        conf.scoped_service(root.prefix(), scope);
    });
}
