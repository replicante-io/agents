use std::sync::Arc;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use opentracingrust::Log;

use replicante_util_actixweb::request_span;
use replicante_util_tracing::fail_span;

use crate::Agent;

/// API interface to Agent::shards
pub struct Shards {
    agent: Arc<dyn Agent>,
}

impl Shards {
    pub fn new(agent: Arc<dyn Agent>) -> Shards {
        Shards { agent }
    }
}

impl Responder for Shards {
    type Error = actix_web::Error;
    type Future = Result<HttpResponse, actix_web::Error>;

    fn respond_to(self, request: &HttpRequest) -> Self::Future {
        let mut exts = request.extensions_mut();
        let mut span = request_span(&mut exts);
        span.log(Log::new().log("span.kind", "server-receive"));
        let shards = self
            .agent
            .shards(&mut span)
            .map_err(|error| fail_span(error, &mut *span))?;
        let response = HttpResponse::Ok().json(shards);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    }
}
