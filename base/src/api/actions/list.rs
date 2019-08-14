use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;

use replicante_util_actixweb::request_span;
use replicante_util_tracing::fail_span;

use crate::AgentContext;

/// List finished actions.
pub fn finished(request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    let actions = context
        .store
        .with_transaction(|tx| {
            let mut actions = Vec::new();
            let iter = tx.actions().finished(span.context().clone())?;
            for action in iter {
                actions.push(action?);
            }
            Ok(actions)
        })
        .map_err(|error| fail_span(error, &mut span))?;
    Ok(HttpResponse::Ok().json(actions))
}

/// List running and pending actions.
pub fn queue(request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    let actions = context
        .store
        .with_transaction(|tx| {
            let mut actions = Vec::new();
            for action in tx.actions().queue(span.context().clone())? {
                actions.push(action?);
            }
            Ok(actions)
        })
        .map_err(|error| fail_span(error, &mut span))?;
    Ok(HttpResponse::Ok().json(actions))
}
