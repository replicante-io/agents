use semver::VersionReq;


mod models;
mod replica;


lazy_static! {
    pub static ref REPLICA_SET_RANGE: VersionReq = VersionReq::parse(">= 3.2.0").unwrap();
}


pub use self::models::BuildInfo;
pub use self::models::ReplSetStatus;
pub use self::replica::ReplicaSet;
