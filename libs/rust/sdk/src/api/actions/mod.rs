use std::sync::Arc;

use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_json::json;

use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::TracingMiddleware;

use crate::actions::actions_enabled;
use crate::actions::ActionDescriptor;
use crate::actions::ACTIONS;
use crate::api::APIRoot;
use crate::AgentContext;

mod action;
mod list;

/// Return a list of available agent actions.
async fn available() -> impl Responder {
    let mut actions: Vec<ActionDescriptor> =
        ACTIONS::iter().map(|action| action.describe()).collect();
    actions.sort_by_key(|action| action.kind.clone());
    HttpResponse::Ok().json(actions)
}

/// Static 2xx response to confirm the actions API is NOT enabled.
async fn index_disabled() -> impl Responder {
    HttpResponse::Ok().json(json!({"actions": false}))
}

/// Static 2xx response to confirm the actions API is enabled.
async fn index_enabled() -> impl Responder {
    HttpResponse::Ok().json(json!({"actions": true}))
}

/// Configure the API server with actions API.
pub fn configure_app(flags: &APIFlags, app: &mut web::ServiceConfig, context: &AgentContext) {
    if actions_enabled(&context.config).unwrap_or(false) {
        configure_enabled(flags, app, context)
    } else {
        configure_disabled(flags, app)
    }
}

/// Configure the API server with actions API disabled.
fn configure_disabled(flags: &APIFlags, app: &mut web::ServiceConfig) {
    APIRoot::UnstableAPI.and_then(flags, |root| {
        app.service(
            root.resource("/actions")
                .route(web::get().to(index_disabled)),
        );
    });
}

/// Configure the API server with actions API enabled.
fn configure_enabled(flags: &APIFlags, app: &mut web::ServiceConfig, context: &AgentContext) {
    APIRoot::UnstableAPI.and_then(flags, |root| {
        let tracer = Arc::clone(&context.tracer);
        app.service(
            root.resource("/actions")
                .route(web::get().to(index_enabled)),
        );
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
            root.resource("/actions/schedule/{kind:.*}")
                .wrap(TracingMiddleware::with_name(
                    context.logger.clone(),
                    Arc::clone(&tracer),
                    "/actions/schedule/{kind}",
                ))
                .route(web::post().to(action::schedule)),
        );
    });
}
