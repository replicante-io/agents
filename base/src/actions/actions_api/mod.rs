use std::sync::Arc;

use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_json::json;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

use super::ActionDescriptor;
use super::ACTIONS;
use crate::api::APIRoot;
use crate::AgentContext;

mod action;
mod list;

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
pub fn configure_app(context: &AgentContext, flags: &APIFlags, app: &mut web::ServiceConfig) {
    APIRoot::UnstableAPI.and_then(flags, |root| {
        let tracer = Arc::clone(&context.tracer);
        app.service(root.resource("/actions").route(web::get().to(index)));
        app.service(
            root.resource("/actions/available")
                .route(web::get().to(available)),
        );
        app.service(
            root.resource("/actions/finished")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(list::finished)),
        );
        app.service(
            root.resource("/actions/info/{id}")
                .wrap(TracingMiddleware::with_name(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                    "/actions/info/{id}",
                ))
                .route(web::get().to(action::info)),
        );
        app.service(
            root.resource("/actions/queue")
                .wrap(TracingMiddleware::new(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                ))
                .route(web::get().to(list::queue)),
        );
        app.service(
            root.resource("/actions/schedule/{kind}")
                .wrap(TracingMiddleware::with_name(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                    "/actions/schedule/{kind}",
                ))
                .route(web::post().to(action::schedule)),
        );
    })
}
