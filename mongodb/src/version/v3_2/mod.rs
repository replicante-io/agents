use semver::VersionReq;

mod common;
mod models;
mod replica;
mod sharded;

lazy_static! {
    pub static ref REPLICA_SET_RANGE: VersionReq = VersionReq::parse(">= 3.2.0").unwrap();
    pub static ref SHARDED_RANGE: VersionReq = VersionReq::parse(">= 3.2.0").unwrap();
}

pub use self::models::BuildInfo;
pub use self::models::ReplSetStatus;
pub use self::replica::ReplicaSet;
pub use self::sharded::Sharded;
