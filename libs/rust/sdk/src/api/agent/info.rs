use std::sync::Arc;

use actix_web::web::Data;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use opentracingrust::Log;

use replicante_util_actixweb::request_span;
use replicante_util_tracing::fail_span;

use crate::Agent;

/// API interface to Agent::agent_info
pub async fn agent(request: HttpRequest, agent: Data<Arc<dyn Agent>>) -> Result<impl Responder> {
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    span.log(Log::new().log("span.kind", "server-receive"));
    let info = agent
        .agent_info(&mut span)
        .map_err(|error| fail_span(error, &mut *span))?;
    let response = HttpResponse::Ok().json(info);
    span.log(Log::new().log("span.kind", "server-send"));
    Ok(response)
}

/// API interface to Agent::datastore_info
pub async fn datastore(
    request: HttpRequest,
    agent: Data<Arc<dyn Agent>>,
    cluster_display_name_override: Data<Option<String>>,
) -> Result<impl Responder> {
    let mut exts = request.extensions_mut();
    let mut span = request_span(&mut exts);
    span.log(Log::new().log("span.kind", "server-receive"));
    let mut info = agent
        .datastore_info(&mut span)
        .map_err(|error| fail_span(error, &mut *span))?;

    // Inject the cluster_display_name override if configured.
    info.cluster_display_name = cluster_display_name_override
        .as_ref()
        .as_ref()
        .cloned()
        .or(info.cluster_display_name);

    let response = HttpResponse::Ok().json(info);
    span.log(Log::new().log("span.kind", "server-send"));
    Ok(response)
}
