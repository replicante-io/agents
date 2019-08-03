use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;

use crate::AgentContext;

/// List finished actions.
pub fn finished(request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let actions = context.store.with_transaction(|tx| {
        let mut actions = Vec::new();
        for action in tx.actions().finished()? {
            actions.push(action?);
        }
        Ok(actions)
    })?;
    Ok(HttpResponse::Ok().json(actions))
}

/// List running and pending actions.
pub fn queue(request: HttpRequest) -> Result<impl Responder> {
    let context = request
        .app_data::<AgentContext>()
        .expect("AgentContext must be available as App::data");
    let actions = context.store.with_transaction(|tx| {
        let mut actions = Vec::new();
        for action in tx.actions().queue()? {
            actions.push(action?);
        }
        Ok(actions)
    })?;
    Ok(HttpResponse::Ok().json(actions))
}
