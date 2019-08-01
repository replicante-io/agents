use failure::ResultExt;
use failure::SyncFailure;
use migrant_lib::Config;
use migrant_lib::Migrator;
use migrant_lib::Settings;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use slog::debug;
use slog::info;
use slog::Logger;

use crate::actions::ActionRecord;
use crate::store::interface::ConnectionImpl;
use crate::store::interface::ConnectionInterface;
use crate::store::interface::PersistImpl;
use crate::store::interface::PersistInterface;
use crate::store::interface::StoreInterface;
use crate::store::interface::TransactionImpl;
use crate::store::interface::TransactionInterface;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

struct Connection {
    connection: PooledConnection<SqliteConnectionManager>,
}

impl ConnectionInterface for Connection {
    fn transaction(&mut self) -> Result<TransactionImpl> {
        let inner = self
            .connection
            .transaction()
            .with_context(|_| ErrorKind::PersistentNoConnection)?;
        let inner = Some(inner);
        Ok(TransactionImpl::new(Transaction { inner }))
    }
}

struct Persist<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
}

impl<'a, 'b: 'a> PersistInterface for Persist<'a, 'b> {
    fn action(&self, action: ActionRecord) -> Result<()> {
        let args = serde_json::to_string(&action.args)
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        let headers = serde_json::to_string(&action.headers)
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        let requester = serde_json::to_string(&action.requester)
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        let state = serde_json::to_string(&action.state)
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        let mut statement = self
            .inner
            .prepare_cached(
                r#"INSERT INTO actions
                    (action, agent_version, args, created_ts, headers, id, requester, state)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        statement
            .execute(&[
                action.action,
                action.agent_version,
                args,
                action.created_ts.to_rfc3339(),
                headers,
                action.id.to_string(),
                requester,
                state,
            ])
            .with_context(|_| ErrorKind::PersistentWrite("new action"))?;
        Ok(())
    }
}

/// SQLite3 backed store.
pub struct Store {
    logger: Logger,
    path: String,
    pool: Pool<SqliteConnectionManager>,
}

impl Store {
    pub fn new(logger: Logger, path: String) -> Result<Store> {
        // Create a connection manager and ensure foreign keys are checked.
        let manager = SqliteConnectionManager::file(&path)
            .with_init(|c| c.execute_batch("PRAGMA foreign_keys=1;"));
        let pool = Pool::builder()
            .build(manager)
            .with_context(|_| ErrorKind::PersistentPool)?;
        Ok(Store { logger, path, pool })
    }
}

impl StoreInterface for Store {
    fn connection(&self) -> Result<ConnectionImpl> {
        let connection = self
            .pool
            .get()
            .with_context(|_| ErrorKind::PersistentNoConnection)?;
        Ok(ConnectionImpl::new(Connection { connection }))
    }

    fn migrate(&self) -> Result<()> {
        debug!(self.logger, "Initialising migrations engine");
        let path = std::env::current_dir()
            .with_context(|_| ErrorKind::PersistentOpen(self.path.clone()))?;
        let path = path.join(&self.path);
        let settings = Settings::configure_sqlite()
            .database_path(path)
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentOpen(self.path.clone()))?
            .build()
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;
        let mut config = Config::with_settings(&settings);
        config
            .setup()
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;
        config.use_cli_compatible_tags(true);

        // Register migrations.
        macro_rules! make_migration {
            ($tag:expr) => {
                migrant_lib::EmbeddedMigration::with_tag($tag)
                    .up(include_str!(concat!("./migrations/", $tag, "/up.sql")))
                    .down(include_str!(concat!("./migrations/", $tag, "/down.sql")))
                    .boxed()
            };
        }
        config
            .use_migrations(&[make_migration!("20190728220141_initialise")])
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;

        info!(self.logger, "Running DB migrations as needed");
        let config = config
            .reload()
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;
        Migrator::with_config(&config)
            .all(true)
            .show_output(true)
            .swallow_completion(true)
            .apply()
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;
        info!(self.logger, "Agent DB ready");
        Ok(())
    }
}

/// Wrap all operations in a SQLite3 transaction.
struct Transaction<'a> {
    inner: Option<rusqlite::Transaction<'a>>,
}

impl<'a> TransactionInterface for Transaction<'a> {
    fn commit(&mut self) -> Result<()> {
        self.inner
            .take()
            .expect("cannot use committed/rolled back transaction")
            .commit()
            .with_context(|_| ErrorKind::PersistentCommit)
            .map_err(Error::from)
    }

    fn persist(&mut self) -> PersistImpl {
        let inner = self
            .inner
            .as_mut()
            .expect("cannot use committed/rolled back transaction");
        let inner = Persist { inner };
        PersistImpl::new(inner)
    }

    fn rollback(&mut self) -> Result<()> {
        self.inner
            .take()
            .expect("cannot use committed/rolled back transaction")
            .rollback()
            .with_context(|_| ErrorKind::PersistentCommit)
            .map_err(Error::from)
    }
}
