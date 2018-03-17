/// Stores agent information.
#[derive(Clone, Debug, Serialize)]
pub struct AgentInfo {
    version: AgentVersion,
}

impl AgentInfo {
    pub fn new(version: AgentVersion) -> AgentInfo {
        AgentInfo { version }
    }
}


/// Stores agent version details.
#[derive(Clone, Debug, Serialize)]
pub struct AgentVersion {
    checkout: String,
    number: String,
    taint: String,
}

impl AgentVersion {
    pub fn new(checkout: &str, number: &str, taint: &str) -> AgentVersion {
        AgentVersion {
            checkout: String::from(checkout),
            number: String::from(number),
            taint: String::from(taint)
        }
    }
}


/// Stores datastore version details.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct DatastoreInfo {
    kind: String,
    version: String,
}

impl DatastoreInfo {
    pub fn new(kind: &str, version: &str) -> DatastoreInfo {
        DatastoreInfo {
            kind: String::from(kind),
            version: String::from(version)
        }
    }
}


/// Stores individual shard information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct Shard {
    id: String,
    role: ShardRole,
    lag: Option<i64>,
    last_op: i64,
}

impl Shard {
    pub fn new(id: &str, role: ShardRole, lag: Option<i64>, last_op: i64) -> Shard {
        Shard {
            id: String::from(id),
            role, lag, last_op
        }
    }
}


/// Enumeration of possible shard roles.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub enum ShardRole {
    Primary,
    Secondary,
    Unknown(String)
}


#[cfg(test)]
mod tests {
    mod agent_info {
        use serde_json;
        use super::super::AgentInfo;
        use super::super::AgentVersion;

        #[test]
        fn to_json() {
            let info = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let payload = serde_json::to_string(&info).unwrap();
            let expected = r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
            assert_eq!(payload, expected);
        }
    }

    mod agent_version {
        use serde_json;
        use super::super::AgentVersion;

        #[test]
        fn to_json() {
            let version = AgentVersion::new("abc123", "1.2.3", "tainted");
            let payload = serde_json::to_string(&version).unwrap();
            let expected = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod datastore_info {
        use serde_json;
        use super::super::DatastoreInfo;

        #[test]
        fn from_json() {
            let payload = r#"{"kind":"DB","version":"1.2.3"}"#;
            let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
            let expected = DatastoreInfo::new("DB", "1.2.3");
            assert_eq!(info, expected);
        }

        #[test]
        fn to_json() {
            let info = DatastoreInfo::new("DB", "1.2.3");
            let payload = serde_json::to_string(&info).unwrap();
            let expected = r#"{"kind":"DB","version":"1.2.3"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod shard {
        use serde_json;
        use super::super::Shard;
        use super::super::ShardRole;

        #[test]
        fn primary_from_json() {
            let payload = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new("shard-1", ShardRole::Primary, Some(0), 12345);
            assert_eq!(shard, expected);
        }

        #[test]
        fn primary_to_json() {
            let shard = Shard::new("shard-1", ShardRole::Primary, Some(0), 12345);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
            assert_eq!(payload, expected);
        }

        #[test]
        fn unkown_from_json() {
            let payload = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new(
                "shard-1", ShardRole::Unknown(String::from("Test")), Some(0), 12345
            );
            assert_eq!(shard, expected);
        }

        #[test]
        fn unkown_to_json() {
            let shard = Shard::new(
                "shard-1", ShardRole::Unknown(String::from("Test")), Some(0), 12345
            );
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
            assert_eq!(payload, expected);
        }

        #[test]
        fn missing_lag_from_json() {
            let payload = r#"{"id":"shard-1","role":"Secondary","last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new("shard-1", ShardRole::Secondary, None, 12345);
            assert_eq!(shard, expected);
        }

        #[test]
        fn no_lag_from_json() {
            let payload = r#"{"id":"shard-1","role":"Secondary","lag":null,"last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new("shard-1", ShardRole::Secondary, None, 12345);
            assert_eq!(shard, expected);
        }

        #[test]
        fn no_lag_to_json() {
            let shard = Shard::new("shard-1", ShardRole::Primary, None, 12345);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"id":"shard-1","role":"Primary","lag":null,"last_op":12345}"#;
            assert_eq!(payload, expected);
        }
    }
}
