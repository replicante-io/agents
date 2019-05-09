use std::collections::HashMap;

use zk_4lw::Error;
use zk_4lw::FourLetterWord;
use zk_4lw::Result;

/// The "conf" command
pub struct Conf;

impl FourLetterWord for Conf {
    type Response = Response;
    fn command() -> &'static str {
        "conf"
    }

    fn parse_response(response: &str) -> Result<Self::Response> {
        let mut zk_server_id: Option<String> = None;
        let mut zk_extras = HashMap::new();

        let lines = response.lines();
        for line in lines {
            let mut iter = line.split('=');
            match (iter.next().map(str::trim), iter.next().map(str::trim)) {
                (Some(key), Some(value)) => match key {
                    "serverId" => zk_server_id = Some(value.into()),
                    _ => {
                        zk_extras.insert(key.into(), value.into());
                    }
                },
                _ => break,
            };
        }

        macro_rules! error_if_none {
            ($($name:ident)*) => {
                $(
                    match $name {
                        Some(v) => v,
                        None => return Err(Error::MissingField(stringify!($name))),
                    }
                )*
            }
        }

        Ok(Response {
            zk_server_id: error_if_none!(zk_server_id),
            zk_extras,
        })
    }
}

/// Sub-set of the "conf" response the agent needs.
pub struct Response {
    pub zk_server_id: String,
    pub zk_extras: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use zk_4lw::FourLetterWord;

    use super::Conf;

    #[test]
    fn parse_valid_response() {
        let response = Conf::parse_response(
            r#"clientPort=2181
dataDir=/data/version-2
dataLogDir=/datalog/version-2
tickTime=2000
maxClientCnxns=60
minSessionTimeout=4000
maxSessionTimeout=40000
serverId=3
initLimit=5
syncLimit=2
electionAlg=3
electionPort=3888
quorumPort=2888
peerType=0"#,
        )
        .unwrap();
        assert_eq!(response.zk_server_id, "3");
        assert_eq!(
            response.zk_extras.get("dataDir").unwrap(),
            "/data/version-2"
        );
        assert_eq!(response.zk_extras.get("minSessionTimeout").unwrap(), "4000");
    }
}
