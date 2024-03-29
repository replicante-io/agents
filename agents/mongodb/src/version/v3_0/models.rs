use mongodb::bson::Timestamp;
use serde::Deserialize;

use replicante_agent::Result;
use replicante_models_agent::info::ShardRole;

use crate::error::ErrorKind;

/// Section of the replSetGetStatus command that we care about.
#[derive(Debug, Deserialize)]
pub struct ReplSetStatus {
    pub members: Vec<ReplSetStatusMember>,
    #[serde(rename = "myState")]
    pub my_state: i32,
    pub set: String,
}

impl ReplSetStatus {
    /// Extracts the timestamp (in seconds) of the latest operation.
    pub fn last_op(&self) -> Result<i64> {
        for member in &self.members {
            if member.is_self {
                return Ok(i64::from(member.optime.time));
            }
        }
        Err(ErrorKind::MembersNoSelf.into())
    }

    /// Extracts the node's name from the output of replSetGetStatus.
    pub fn node_name(&self) -> Result<String> {
        for member in &self.members {
            if member.is_self {
                return Ok(member.name.clone());
            }
        }
        Err(ErrorKind::MembersNoSelf.into())
    }

    /// Extracts the optime (in seconds) of the primary.
    pub fn primary_optime(&self) -> Result<i64> {
        for member in &self.members {
            if member.state == 1 {
                return Ok(i64::from(member.optime.time));
            }
        }
        Err(ErrorKind::MembersNoPrimary.into())
    }

    /// Extracts the node's role in the Replica Set.
    pub fn role(&self) -> Result<ShardRole> {
        match self.my_state {
            0 => Ok(ShardRole::Unknown(String::from("STARTUP"))),
            1 => Ok(ShardRole::Primary),
            2 => Ok(ShardRole::Secondary),
            3 => Ok(ShardRole::Unknown(String::from("RECOVERING"))),
            5 => Ok(ShardRole::Unknown(String::from("STARTUP2"))),
            6 => Ok(ShardRole::Unknown(String::from("UNKNOWN"))),
            7 => Ok(ShardRole::Unknown(String::from("ARBITER"))),
            8 => Ok(ShardRole::Unknown(String::from("DOWN"))),
            9 => Ok(ShardRole::Unknown(String::from("ROLLBACK"))),
            10 => Ok(ShardRole::Unknown(String::from("REMOVED"))),
            state => Err(ErrorKind::UnsupportedSateId(state).into()),
        }
    }
}

/// Section of the replSetGetStatus member that we care about.
#[derive(Debug, Deserialize)]
pub struct ReplSetStatusMember {
    #[serde(rename = "self", default = "ReplSetStatusMember::default_self")]
    pub is_self: bool,
    pub name: String,
    pub optime: Timestamp,
    pub state: i32,
}

impl ReplSetStatusMember {
    fn default_self() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use mongodb::bson;
    use mongodb::bson::doc;
    use mongodb::bson::Bson;
    use mongodb::bson::Timestamp;

    use replicante_agent::ErrorKind;
    use replicante_models_agent::info::ShardRole;

    use super::ReplSetStatus;

    lazy_static! {
        static ref MONGO_TIMESTAMP_ONE: Bson = {
            let time = 1514677701_u32.to_le();
            let ts = Timestamp { time, increment: 0 };
            Bson::Timestamp(ts)
        };
        static ref MONGO_TIMESTAMP_TWO: Bson = {
            let time = 1514677698_u32.to_le();
            let ts = Timestamp { time, increment: 0 };
            Bson::Timestamp(ts)
        };
    }

    fn make_rs() -> Bson {
        Bson::Document(doc! {
            "set": "test-rs",
            "members": [{
                "_id": 0,
                "name": "host0",
                "optime": MONGO_TIMESTAMP_ONE.clone(),
                "self": false,
                "state": 1,
            }, {
                "_id": 1,
                "name": "host1",
                "optime": MONGO_TIMESTAMP_TWO.clone(),
                "self": true,
                "state": 2,
            }],
            "myState": 1,
        })
    }

    #[test]
    fn last_op() {
        let rs: ReplSetStatus = bson::from_bson(make_rs()).unwrap();
        let last_op = rs.last_op().unwrap();
        assert_eq!(last_op, 1514677698);
    }

    #[test]
    fn last_op_without_self_fails() {
        let rs = Bson::Document(doc! {
            "set": "test-rs",
            "members": [{
                "_id": 0,
                "name": "host0",
                "optime": MONGO_TIMESTAMP_ONE.clone(),
                "self": false,
                "state": 2,
            }],
            "myState": 1,
        });
        let rs: ReplSetStatus = bson::from_bson(rs).unwrap();
        match rs.last_op() {
            Err(error) => match error.kind() {
                &ErrorKind::InvalidStoreState(ref msg) => {
                    assert_eq!("self not in members list", msg.to_string());
                }
                _ => panic!("Unexpected error {:?}", error),
            },
            Ok(result) => panic!("Unexpected success {:?}", result),
        };
    }

    #[test]
    fn node_name() {
        let rs: ReplSetStatus = bson::from_bson(make_rs()).unwrap();
        let node_name = rs.node_name().unwrap();
        assert_eq!("host1", node_name);
    }

    #[test]
    fn node_name_without_self_fails() {
        let rs = Bson::Document(doc! {
            "set": "test-rs",
            "members": [{
                "_id": 0,
                "name": "host0",
                "optime": MONGO_TIMESTAMP_ONE.clone(),
                "self": false,
                "state": 2,
            }],
            "myState": 1,
        });
        let rs: ReplSetStatus = bson::from_bson(rs).unwrap();
        match rs.node_name() {
            Err(error) => match error.kind() {
                &ErrorKind::InvalidStoreState(ref msg) => {
                    assert_eq!("self not in members list", msg.to_string());
                }
                _ => panic!("Unexpected error {:?}", error),
            },
            Ok(result) => panic!("Unexpected success {:?}", result),
        };
    }

    #[test]
    fn primary_optime() {
        let rs: ReplSetStatus = bson::from_bson(make_rs()).unwrap();
        let primary_optime = rs.primary_optime().unwrap();
        assert_eq!(1514677701, primary_optime);
    }

    #[test]
    fn primary_optime_without_primary() {
        let rs = Bson::Document(doc! {
            "set": "test-rs",
            "members": [{
                "_id": 0,
                "name": "host0",
                "optime": MONGO_TIMESTAMP_ONE.clone(),
                "self": false,
                "state": 2,
            }],
            "myState": 1,
        });
        let rs: ReplSetStatus = bson::from_bson(rs).unwrap();
        match rs.primary_optime() {
            Err(error) => match error.kind() {
                &ErrorKind::InvalidStoreState(ref msg) => {
                    assert_eq!("primary node not in members list", msg.to_string());
                }
                _ => panic!("Unexpected error {:?}", error),
            },
            Ok(result) => panic!("Unexpected success {:?}", result),
        };
    }

    #[test]
    fn role_primary() {
        let rs = Bson::Document(doc! {
            "set": "test-rs",
            "members": [],
            "myState": 1,
        });
        let rs: ReplSetStatus = bson::from_bson(rs).unwrap();
        let role = rs.role().unwrap();
        assert_eq!(ShardRole::Primary, role);
    }

    #[test]
    fn role_not_supported() {
        let rs = Bson::Document(doc! {
            "set": "test-rs",
            "members": [],
            "myState": 22,
        });
        let rs: ReplSetStatus = bson::from_bson(rs).unwrap();
        match rs.role() {
            Err(error) => match error.kind() {
                &ErrorKind::InvalidStoreState(ref msg) => {
                    assert_eq!("unsupported node state 22", msg.to_string());
                }
                _ => panic!("Unexpected error {:?}", error),
            },
            Ok(result) => panic!("Unexpected success {:?}", result),
        };
    }
}
