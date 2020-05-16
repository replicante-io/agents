use actix_web::web;

use replicante_util_actixweb::RootDescriptor;

mod info;
mod shards;

use crate::api::APIRoot;
use crate::api::AppConfigContext;

/// Configure all agent endpoints.
pub fn configure(conf: &mut AppConfigContext) {
    APIRoot::UnstableAPI.and_then(&conf.context.flags, |root| {
        let agent = self::info::agent(&conf.context.agent);
        let datastore = self::info::datastore(&conf.context.agent);
        let shards = self::shards::shards(&conf.context.agent);
        let scope = web::scope("/info").service(agent).service(datastore);
        let prefix = root.prefix();
        conf.scoped_service(prefix, scope);
        conf.scoped_service(prefix, shards);
    });
}
