use mongodb;

use replicante_agent::AgentError;


#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn to_agent(error: mongodb::error::Error) -> AgentError {
    match error {
        _ => AgentError::DatastoreError(error.to_string())
    }
}


#[cfg(test)]
mod tests {
    use mongodb::error::Error;
    use replicante_agent::AgentError;
    use super::to_agent;

    #[test]
    fn operational_error_conversion() {
        let err = Error::OperationError(String::from("abc"));
        match to_agent(err) {
            AgentError::DatastoreError(msg) => assert_eq!(msg, "abc"),
            _ => panic!("Error is not of valid type")
        }
    }
}
