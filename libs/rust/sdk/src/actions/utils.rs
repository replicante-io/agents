use serde::de::DeserializeOwned;
use serde_json::Value as Json;

use crate::actions::ActionValidity;
use crate::actions::ActionValidityError;

/// Validate the JSON arguments can be decoded in the given type T.
pub fn validate_action_args<T>(args: Json) -> ActionValidity<T>
where
    T: DeserializeOwned,
{
    match serde_json::from_value(args) {
        Ok(args) => Ok(args),
        Err(error) => Err(ActionValidityError::InvalidArgs(error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::Deserialize;
    use serde_json::json;

    use crate::actions::ActionValidity;
    use crate::actions::ActionValidityError;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct TestArgs {
        a: String,
        b: bool,
    }

    #[test]
    fn args_not_valid() {
        let args = json!({"b": true});
        let args: ActionValidity<TestArgs> = super::validate_action_args(args);
        match args {
            Err(ActionValidityError::InvalidArgs(_)) => (),
            other => panic!("unexpected value: {:?}", other),
        }
    }

    #[test]
    fn args_valid() {
        let args = json!({
            "a": "c",
            "b": true,
        });
        let args: TestArgs = super::validate_action_args(args).unwrap();
        assert_eq!(
            args,
            TestArgs {
                a: "c".into(),
                b: true,
            }
        );
    }
}
