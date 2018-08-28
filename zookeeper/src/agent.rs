use opentracingrust::Log;
use opentracingrust::Span;
use opentracingrust::StartOptions;
use opentracingrust::utils::FailSpan;

use zk_4lw::Client;
use zk_4lw::FourLetterWord;

use replicante_agent::Agent;
use replicante_agent::AgentContext;
use replicante_agent::Result;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::Config;
use super::errors::to_agent;
use super::zk4lw::Conf;
use super::zk4lw::Srvr;


lazy_static! {
    pub static ref AGENT_VERSION: AgentVersion = AgentVersion::new(
        env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_TAINT")
    );
}


/// Converts a Zookeeper version into a Semver compatible string.
///
/// In particular it reformats the commit hash as metadata.
fn to_semver(version: String) -> Result<String> {
    let ver = version.split(',').next()
        .expect("splitting version string returned no elements");
    let mut iter = ver.split('-');
    match (iter.next().map(|s| s.trim()), iter.next().map(|s| s.trim())) {
        (Some(version), Some(hash)) => Ok(format!("{}+{}", version, hash)),
        (Some(version), None) => Ok(version.into()),
        _ => Err(format!("Unable to parse version: {}", version).into())
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
        let mut span = self.agent_context.tracer.span_with_options(
            "conf", StartOptions::default().child_of(root.context().clone())
        ).auto_finish();
        span.log(Log::new().log("span.kind", "client-send"));
        let conf = self.zk_client.exec::<Conf>().map_err(to_agent).fail_span(&mut span);
        span.log(Log::new().log("span.kind", "client-receive"));
        conf
    }

    /// Executes the "conf" 4lw against the zookeeper server.
    fn srvr(&self, root: &Span) -> Result<<Srvr as FourLetterWord>::Response> {
        let mut span = self.agent_context.tracer.span_with_options(
            "srvr", StartOptions::default().child_of(root.context().clone())
        ).auto_finish();
        span.log(Log::new().log("span.kind", "client-send"));
        let srvr = self.zk_client.exec::<Srvr>().map_err(to_agent).fail_span(&mut span);
        span.log(Log::new().log("span.kind", "client-receive"));
        srvr
    }
}

impl Agent for ZookeeperAgent {
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        span.log(Log::new().log("span.kind", "server-receive"));
        let info = AgentInfo::new(AGENT_VERSION.clone());
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(info)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        span.log(Log::new().log("span.kind", "server-receive"));
        let name = self.conf(span)?.zk_server_id;
        let version = to_semver(self.srvr(span)?.zk_version)?;
        let info = DatastoreInfo::new(self.cluster_name.clone(), "Zookeeper", name, version);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(info)
    }

    fn shards(&self, _span: &mut Span) -> Result<Shards> {
        Err("TODO".into())
    }
}


#[cfg(test)]
mod tests {
    use super::to_semver;

    #[test]
    fn conver_to_semver() {
        let version = to_semver(
            "3.4.13-2d71af4dbe22557fda74f9a9b4309b15a7487f03, built on 06/29/2018 04:05 GMT".into()
        ).unwrap();
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
