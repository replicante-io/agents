use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_json::json;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;

use super::ActionDescriptor;
use super::ACTIONS;
use crate::api::APIRoot;

/// Return a list of available agent actions.
fn available() -> impl Responder {
    let actions: Vec<ActionDescriptor> = ACTIONS::iter().map(|action| action.describe()).collect();
    HttpResponse::Ok().json(actions)
}

/// Static 2xx response to confirm the actions API is enabled.
fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({"actions": true}))
}

/// Configure the API server with actions API.
pub fn configure_app(flags: &APIFlags, app: &mut web::ServiceConfig) {
    APIRoot::UnstableAPI.and_then(flags, |root| {
        app.service(root.resource("/actions").route(web::get().to(index)));
        app.service(
            root.resource("/actions/available")
                .route(web::get().to(available)),
        );
    })
}
