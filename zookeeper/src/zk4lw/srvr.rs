use std::collections::HashMap;

use zk_4lw::Error;
use zk_4lw::FourLetterWord;
use zk_4lw::Result;

/// The "srvr" command
pub struct Srvr;

impl FourLetterWord for Srvr {
    type Response = Response;
    fn command() -> &'static str { "srvr" }

    fn parse_response(response: &str) -> Result<Self::Response> {
        let mut zk_version: Option<String> = None;
        let mut zk_extras = HashMap::new();

        let lines = response.lines();
        for line in lines {
            let mut iter = line.split(':');
            match (iter.next().map(|s| s.trim()), iter.next().map(|s| s.trim())) {
                (Some(key), Some(value)) => {
                    match key {
                        "Zookeeper version" => zk_version = Some(value.into()),
                        _ => { zk_extras.insert(key.into(), value.into()); },
                    }
                },
                _ => break
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
            zk_version: error_if_none!(zk_version),
            zk_extras,
        })
    }
}


/// Sub-set of the "srvr" response the agent needs.
pub struct Response {
    pub zk_version: String,
    pub zk_extras: HashMap<String, String>,
}


#[cfg(test)]
mod tests {
    use zk_4lw::FourLetterWord;
    use super::Srvr;

    #[test]
    fn parse_valid_response() {
        let response = Srvr::parse_response(r#"Zookeeper version: 3.4.13-2d71af4dbe22557fda74f9a9b4309b15a7487f03, built on 06/29/2018 04:05 GMT                                                                              
Latency min/avg/max: 0/0/0
Received: 8
Sent: 7
Connections: 1
Outstanding: 0
Zxid: 0x600000004
Mode: leader
Node count: 4
Proposal sizes last/min/max: 32/32/36"#).unwrap();
        //assert_eq!(response.zk_server_id, "3");
        assert_eq!(response.zk_extras.get("Latency min/avg/max").unwrap(), "0/0/0");
        assert_eq!(response.zk_extras.get("Proposal sizes last/min/max").unwrap(), "32/32/36");
    }
}
