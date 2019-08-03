use std::str::FromStr;

use chrono::TimeZone;
use chrono::Utc;
use failure::ResultExt;
use rusqlite::params;
use uuid::Uuid;

use crate::actions::ActionRecord;
use crate::store::interface::ActionInterface;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

const ACTION_GET: &str = "action.get";

/// Helper macro to avoid writing the same match every time.
macro_rules! decode_or_return {
    ($decode:expr, $op:expr $(,)?) => {
        match $decode {
            Ok(r) => r,
            Err(error) => {
                let error = Err(error)
                    .with_context(|_| ErrorKind::PersistentRead($op))
                    .map_err(Error::from);
                return error;
            }
        }
    };
}

pub struct Action<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
}

impl<'a, 'b: 'a> Action<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>) -> Action<'a, 'b> {
        Action { inner }
    }
}

impl<'a, 'b: 'a> ActionInterface for Action<'a, 'b> {
    fn get(&self, id: &str) -> Result<Option<ActionRecord>> {
        let mut statement = self
            .inner
            .prepare_cached(
                r#"SELECT
                        action,
                        agent_version,
                        args,
                        created_ts,
                        headers,
                        id,
                        requester,
                        state
                    FROM actions
                    WHERE id = ?;
                "#,
            )
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))?;
        let mut rows = statement
            .query(params![id])
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))?;
        let row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))?;
        let row = match row {
            None => return Ok(None),
            Some(row) => row,
        };
        let id: String = decode_or_return!(row.get("id"), ACTION_GET);
        let id = decode_or_return!(Uuid::from_str(&id), ACTION_GET);
        let action: String = decode_or_return!(row.get("action"), ACTION_GET);
        let agent_version: String = decode_or_return!(row.get("agent_version"), ACTION_GET);
        let args: String = decode_or_return!(row.get("args"), ACTION_GET);
        let args = decode_or_return!(serde_json::from_str(&args), ACTION_GET);
        let created_ts: i64 = decode_or_return!(row.get("created_ts"), ACTION_GET);
        let created_ts = Utc.timestamp(created_ts, 0);
        let headers: String = decode_or_return!(row.get("headers"), ACTION_GET);
        let headers = decode_or_return!(serde_json::from_str(&headers), ACTION_GET);
        let requester: String = decode_or_return!(row.get("requester"), ACTION_GET);
        let requester = decode_or_return!(serde_json::from_str(&requester), ACTION_GET);
        let state: String = decode_or_return!(row.get("state"), ACTION_GET);
        let state = decode_or_return!(serde_json::from_str(&state), ACTION_GET);
        Ok(Some(ActionRecord {
            action,
            agent_version,
            args,
            created_ts,
            headers,
            id,
            requester,
            state,
        }))
    }
}
