use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::StartOptions;

use failure::ResultExt;
use zk_4lw::Client;
use zk_4lw::FourLetterWord;

use replicante_agent::fail_span;
use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::CommitOffset;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shard;
use replicante_agent_models::ShardRole;
use replicante_agent_models::Shards;

use super::error::ErrorKind;
use super::metrics::OPS_COUNT;
use super::metrics::OPS_DURATION;
use super::metrics::OP_ERRORS_COUNT;
use super::zk4lw::Conf;
use super::zk4lw::Srvr;
use super::Config;

lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_TAINT")
    );
}

/// Converts a Zookeeper version into a Semver compatible string.
///
/// In particular it reformats the commit hash as metadata.
fn to_semver(version: &str) -> Result<String> {
    let ver = version
        .split(',')
        .next()
        .expect("splitting version string returned no elements");
    let mut iter = ver.split('-');
    match (iter.next().map(str::trim), iter.next().map(str::trim)) {
        (Some(version), Some(hash)) => Ok(format!("{}+{}", version, hash)),
        (Some(version), None) => Ok(version.into()),
        _ => Err(ErrorKind::VersionParse.into()),
    }
}

/// Zookeeper 3.3+ agent.
pub struct ZookeeperAgent {
    agent_context: AgentContext,
    cluster_name: String,
    zk_client: Client,
}

impl ZookeeperAgent {
    pub fn new(config: Config, context: AgentContext) -> ZookeeperAgent {
        ZookeeperAgent {
            agent_context: context,
            cluster_name: config.zookeeper.cluster,
            zk_client: Client::new(config.zookeeper.target),
        }
    }

    /// Executes the "conf" 4lw against the zookeeper server.
    fn conf(&self, root: &Span) -> Result<<Conf as FourLetterWord>::Response> {
        let mut span = self
            .agent_context
            .tracer
            .span_with_options(
                "conf",
                StartOptions::default().child_of(root.context().clone()),
            )
            .auto_finish();
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["conf"]).inc();
        let timer = OPS_DURATION.with_label_values(&["conf"]).start_timer();
        let conf = self
            .zk_client
            .exec::<Conf>()
            .map_err(|error| {
                OP_ERRORS_COUNT.with_label_values(&["conf"]).inc();
                fail_span(error, &mut span)
            })
            .with_context(|_| ErrorKind::StoreOpFailed("conf"))?;
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(conf)
    }

    /// Executes the "conf" 4lw against the zookeeper server.
    fn srvr(&self, root: &Span) -> Result<<Srvr as FourLetterWord>::Response> {
        let mut span = self
            .agent_context
            .tracer
            .span_with_options(
                "srvr",
                StartOptions::default().child_of(root.context().clone()),
            )
            .auto_finish();
        span.log(Log::new().log("span.kind", "client-send"));
        OPS_COUNT.with_label_values(&["srvr"]).inc();
        let timer = OPS_DURATION.with_label_values(&["srvr"]).start_timer();
        let srvr = self
            .zk_client
            .exec::<Srvr>()
            .map_err(|error| {
                OP_ERRORS_COUNT.with_label_values(&["srvr"]).inc();
                fail_span(error, &mut span)
            })
            .with_context(|_| ErrorKind::StoreOpFailed("srvr"))?;
        timer.observe_duration();
        span.log(Log::new().log("span.kind", "client-receive"));
        Ok(srvr)
    }
}

impl Agent for ZookeeperAgent {
    fn agent_info(&self, _: &mut Span) -> Result<AgentInfo> {
        let info = AgentInfo::new(AGENT_VERSION.clone());
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let name = self.conf(span)?.zk_server_id;
        let version = to_semver(&self.srvr(span)?.zk_version)?;
        let info = DatastoreInfo::new(self.cluster_name.clone(), "Zookeeper", name, version, None);
        Ok(info)
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        let srvr = self.srvr(span)?;
        let role = match srvr.zk_mode.as_ref() {
            "leader" => ShardRole::Primary,
            "follower" => ShardRole::Secondary,
            unkown => ShardRole::Unknown(unkown.into()),
        };
        let commit_offset = CommitOffset::unit(srvr.zk_zxid, "zxid");
        let commit_offset = Some(commit_offset);
        let shard = Shard::new(self.cluster_name.clone(), role, commit_offset, None);
        let shards = Shards::new(vec![shard]);
        Ok(shards)
    }
}

#[cfg(test)]
mod tests {
    use super::to_semver;

    #[test]
    fn conver_to_semver() {
        let version = to_semver(
            "3.4.13-2d71af4dbe22557fda74f9a9b4309b15a7487f03, built on 06/29/2018 04:05 GMT".into(),
        )
        .unwrap();
        assert_eq!(version, "3.4.13+2d71af4dbe22557fda74f9a9b4309b15a7487f03");
    }

    #[test]
    fn conver_to_semver_empty() {
        let version = to_semver("".into()).unwrap();
        assert_eq!(version, "");
    }

    #[test]
    fn conver_to_semver_junk() {
        let version = to_semver("abc -def-123, some, other, string".into()).unwrap();
        assert_eq!(version, "abc+def");
    }

    #[test]
    fn conver_to_semver_without_commit() {
        let version = to_semver("3.4.13, built on 06/29/2018 04:05 GMT".into()).unwrap();
        assert_eq!(version, "3.4.13");
    }

    #[test]
    fn conver_to_semver_without_commit_or_date() {
        let version = to_semver("3.4.13".into()).unwrap();
        assert_eq!(version, "3.4.13");
    }

    #[test]
    fn conver_to_semver_without_date() {
        let version = to_semver("3.4.13-2d71af4dbe22557fda74f9a9b4309b15a7487f03".into()).unwrap();
        assert_eq!(version, "3.4.13+2d71af4dbe22557fda74f9a9b4309b15a7487f03");
    }
}
