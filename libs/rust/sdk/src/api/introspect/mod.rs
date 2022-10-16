use actix_web::dev::HttpServiceFactory;
use actix_web::web;

use replicante_util_actixweb::MetricsExporter;
use replicante_util_actixweb::RootDescriptor;

use crate::api::APIRoot;
use crate::api::AppConfigContext;
use crate::AgentContext;

mod threads;

/// Configure all introspection endpoints.
pub fn configure(conf: &mut AppConfigContext) {
    APIRoot::UnstableIntrospect.and_then(&conf.context.flags, |root| {
        let metrics = metrics(&conf.context.agent);
        let prefix = root.prefix();
        conf.scoped_service(prefix, metrics);
        conf.scoped_service(prefix, self::threads::responder);
    });
}

fn metrics(context: &AgentContext) -> impl HttpServiceFactory {
    let registry = context.metrics.clone();
    let metrics = MetricsExporter::with_registry(registry);
    web::resource("/metrics").route(web::get().to(metrics))
}
