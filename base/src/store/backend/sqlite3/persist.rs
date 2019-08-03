use failure::ResultExt;
use rusqlite::params;

use crate::actions::ActionRecord;
use crate::store::interface::PersistInterface;
use crate::ErrorKind;
use crate::Result;

const PERSIST_ACTION: &str = "persist.action";

pub struct Persist<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
}

impl<'a, 'b: 'a> Persist<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>) -> Persist<'a, 'b> {
        Persist { inner }
    }
}

impl<'a, 'b: 'a> PersistInterface for Persist<'a, 'b> {
    fn action(&self, action: ActionRecord) -> Result<()> {
        let args = serde_json::to_string(&action.args)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let headers = serde_json::to_string(&action.headers)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let requester = serde_json::to_string(&action.requester)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let state = serde_json::to_string(&action.state)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let mut statement = self
            .inner
            .prepare_cached(
                r#"INSERT INTO actions
                    (action, agent_version, args, created_ts, headers, id, requester, state)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        statement
            .execute(params![
                action.action,
                action.agent_version,
                args,
                action.created_ts.timestamp(),
                headers,
                action.id.to_string(),
                requester,
                state,
            ])
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        Ok(())
    }
}
