use std::fmt::Display;

use replicante_agent::Error;


#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn to_agent<E: Display>(error: E) -> Error {
    error.to_string().into()
}


#[cfg(test)]
mod tests {
    use jmx::Error;
    use replicante_agent::Error as AgentError;
    use replicante_agent::ErrorKind;
    use super::to_agent;

    #[test]
    fn error_conversion() {
        let err: Error = "abc".into();
        match to_agent(err) {
            AgentError(ErrorKind::Msg(msg), _) => assert_eq!(msg, "abc"),
            _ => panic!("Error is not of valid type")
        }
    }
}
