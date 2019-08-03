use std::str::FromStr;

use failure::ResultExt;
use rusqlite::Statement;
use rusqlite::NO_PARAMS;
use uuid::Uuid;

use crate::actions::ActionListItem;
use crate::actions::ActionState;
use crate::store::interface::ActionsInterface;
use crate::store::Iter;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

const ACTIONS_FINISHED: &str = "action.finished";
const ACTIONS_QUEUE: &str = "action.queue";

/// Helper macro to avoid writing the same match every time.
macro_rules! decode_or_continue {
    ($decode:expr, $res:ident, $op:expr $(,)?) => {
        match $decode {
            Ok(r) => r,
            Err(error) => {
                let error = Err(error)
                    .with_context(|_| ErrorKind::PersistentRead($op))
                    .map_err(Error::from);
                $res.push(error);
                continue;
            }
        }
    };
}

/// Helper to convert the result of a SELECT id, state ...; into an ActionListItem iterator.
fn parse_actions_list(statement: &mut Statement, op: &'static str) -> Result<Iter<ActionListItem>> {
    let mut results = Vec::new();
    let mut rows = statement
        .query(NO_PARAMS)
        .with_context(|_| ErrorKind::PersistentRead(op))?;
    let mut maybe_row = rows
        .next()
        .with_context(|_| ErrorKind::PersistentRead(op))?;
    while let Some(row) = maybe_row {
        let id: String = decode_or_continue!(row.get("id"), results, op);
        let id = decode_or_continue!(Uuid::from_str(&id), results, op);
        let action: String = decode_or_continue!(row.get("action"), results, op);
        let state: String = decode_or_continue!(row.get("state"), results, op);
        let state: ActionState = decode_or_continue!(serde_json::from_str(&state), results, op);
        results.push(Ok(ActionListItem { action, id, state }));
        maybe_row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(op))?;
    }
    Ok(Iter::new(results.into_iter()))
}

pub struct Actions<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
}

impl<'a, 'b: 'a> Actions<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>) -> Actions<'a, 'b> {
        Actions { inner }
    }
}

impl<'a, 'b: 'a> ActionsInterface for Actions<'a, 'b> {
    fn finished(&self) -> Result<Iter<ActionListItem>> {
        let mut statement = self
            .inner
            .prepare_cached(
                r#"SELECT action, id, state FROM actions
                    WHERE state == '"SUCCESS"'
                    OR state == '"FAILED"'
                    OR state == '"CANCELLED"'
                    ORDER BY created_ts ASC
                    -- Limit result as a form of blast radius containment from bugs or overload.
                    -- There really should not be many finished actions still on the agent DB.
                    LIMIT 100;
                "#,
            )
            .with_context(|_| ErrorKind::PersistentRead(ACTIONS_FINISHED))?;
        parse_actions_list(&mut statement, ACTIONS_FINISHED)
    }

    fn queue(&self) -> Result<Iter<ActionListItem>> {
        let mut statement = self
            .inner
            .prepare_cached(
                r#"SELECT action, id, state FROM actions
                    WHERE state != '"SUCCESS"'
                    AND state != '"FAILED"'
                    AND state != '"CANCELLED"'
                    ORDER BY created_ts ASC
                    -- Limit result as a form of blast radius containment in case of bugs.
                    -- There really should not be many running/pending actions on an agent.
                    LIMIT 100;
                "#,
            )
            .with_context(|_| ErrorKind::PersistentRead(ACTIONS_QUEUE))?;
        parse_actions_list(&mut statement, ACTIONS_QUEUE)
    }
}
