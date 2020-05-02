use std::sync::Arc;

use actix_web::web::Data;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use opentracingrust::Log;

use replicante_util_actixweb::request_span;
use replicante_util_tracing::fail_span;

use crate::Agent;
use crate::Result;

/// API interface to Agent::shards
pub async fn shards(request: HttpRequest, agent: Data<Arc<dyn Agent>>) -> Result<impl Responder> {
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    span.log(Log::new().log("span.kind", "server-receive"));
    let shards = agent
        .shards(&mut span)
        .map_err(|error| fail_span(error, &mut *span))?;
    let response = HttpResponse::Ok().json(shards);
    span.log(Log::new().log("span.kind", "server-send"));
    Ok(response)
}
