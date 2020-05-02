use failure::ResultExt;
use failure::SyncFailure;
use migrant_lib::Config;
use migrant_lib::Migrator;
use migrant_lib::Settings;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_util_tracing::MaybeTracer;

use crate::metrics::SQLITE_CONNECTION_ERRORS;
use crate::metrics::SQLITE_OPS_COUNT;
use crate::metrics::SQLITE_OPS_DURATION;
use crate::metrics::SQLITE_OP_ERRORS_COUNT;
use crate::store::interface::ActionImpl;
use crate::store::interface::ActionsImpl;
use crate::store::interface::ConnectionImpl;
use crate::store::interface::ConnectionInterface;
use crate::store::interface::StoreInterface;
use crate::store::interface::TransactionImpl;
use crate::store::interface::TransactionInterface;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

mod action;
mod actions;

struct Connection {
    connection: rusqlite::Connection,
    tracer: MaybeTracer,
}

impl Connection {
    fn new(path: &str, tracer: MaybeTracer) -> Result<Connection> {
        let connection = rusqlite::Connection::open_with_flags(path, Default::default())
            .with_context(|_| ErrorKind::PersistentPool)?;
        // Ensure foreign keys are checked.
        connection
            .execute_batch("PRAGMA foreign_keys=1;")
            .with_context(|_| ErrorKind::PersistentPool)?;
        Ok(Connection { connection, tracer })
    }
}

impl ConnectionInterface for Connection {
    fn transaction(&mut self) -> Result<TransactionImpl> {
        SQLITE_OPS_COUNT.with_label_values(&["BEGIN"]).inc();
        let timer = SQLITE_OPS_DURATION
            .with_label_values(&["BEGIN"])
            .start_timer();
        let inner = self
            .connection
            .transaction()
            .with_context(|_| ErrorKind::PersistentNoConnection)
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["BEGIN"]).inc();
                error
            })?;
        timer.observe_duration();
        let inner = Some(inner);
        let tracer = self.tracer.clone();
        Ok(TransactionImpl::new(Transaction { inner, tracer }))
    }
}

/// SQLite3 backed store.
pub struct Store {
    logger: Logger,
    path: String,
    tracer: MaybeTracer,
}

impl Store {
    pub fn new(logger: Logger, path: String, tracer: MaybeTracer) -> Result<Store> {
        Ok(Store {
            logger,
            path,
            tracer,
        })
    }
}

impl StoreInterface for Store {
    fn connection(&self) -> Result<ConnectionImpl> {
        let tracer = self.tracer.clone();
        let connection = Connection::new(&self.path, tracer).map_err(|error| {
            SQLITE_CONNECTION_ERRORS.inc();
            error
        })?;
        Ok(ConnectionImpl::new(connection))
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
    tracer: MaybeTracer,
}

impl<'a> Transaction<'a> {
    fn tx(&self) -> &rusqlite::Transaction<'a> {
        self.inner
            .as_ref()
            .expect("cannot use committed/rolled back transaction")
    }
}

impl<'a> TransactionInterface for Transaction<'a> {
    fn action(&mut self) -> ActionImpl {
        let inner = self.tx();
        let inner = self::action::Action::new(inner, self.tracer.clone());
        ActionImpl::new(inner)
    }

    fn actions(&mut self) -> ActionsImpl {
        let inner = self.tx();
        let inner = self::actions::Actions::new(inner, self.tracer.clone());
        ActionsImpl::new(inner)
    }

    fn commit(&mut self) -> Result<()> {
        SQLITE_OPS_COUNT.with_label_values(&["COMMIT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["COMMIT"])
            .start_timer();
        self.inner
            .take()
            .expect("cannot use committed/rolled back transaction")
            .commit()
            .with_context(|_| ErrorKind::PersistentCommit)
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["COMMIT"]).inc();
                Error::from(error)
            })
    }

    fn rollback(&mut self) -> Result<()> {
        SQLITE_OPS_COUNT.with_label_values(&["ROLLBACK"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["ROLLBACK"])
            .start_timer();
        self.inner
            .take()
            .expect("cannot use committed/rolled back transaction")
            .rollback()
            .with_context(|_| ErrorKind::PersistentCommit)
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT
                    .with_label_values(&["ROLLBACK"])
                    .inc();
                Error::from(error)
            })
    }
}
