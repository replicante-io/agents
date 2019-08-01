use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

mod backend;
mod interface;

pub use self::backend::backend_factory;

use self::interface::PersistImpl;
use self::interface::StoreImpl;
use self::interface::TransactionImpl;
use crate::actions::ActionRecord;
use crate::Result;

/// Interface to persist data to the store.
pub struct Persist<'a> {
    inner: PersistImpl<'a>,
}

impl<'a> Persist<'a> {
    /// Persist a NEW action to the store.
    pub fn action(&self, action: ActionRecord) -> Result<()> {
        self.inner.action(action)
    }
}

/// Interface to the agent's persistent storage.
#[derive(Clone)]
pub struct Store {
    logger: Logger,
    inner: StoreImpl,
}

impl Store {
    /// Perform database initialisation and applies migrations.
    ///
    /// This method requires a mutable borrow to ensure it can only
    /// be called during the process initialisation phase.
    pub fn migrate(&mut self) -> Result<()> {
        self.inner.migrate()
    }

    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock() -> Store {
        let inner = self::backend::mock::MockStore::new();
        let inner = StoreImpl::new(inner);
        let logger = Logger::root(slog::Discard, slog::o!());
        Store { inner, logger }
    }

    pub fn with_transaction<F, T>(&self, block: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction) -> Result<T>,
    {
        let mut connection = self.inner.connection()?;
        let tx = connection.transaction()?;
        let mut tx = Transaction { inner: tx };
        match block(&mut tx) {
            Err(error) => {
                if let Err(error) = tx.rollback() {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Failed to rollback failed transaction";
                        failure_info(&error),
                    );
                }
                Err(error)
            }
            Ok(rv) => {
                tx.commit()?;
                Ok(rv)
            }
        }
    }
}

/// Interface to transactional operations on the store.
pub struct Transaction<'a> {
    inner: TransactionImpl<'a>,
}

impl<'a> Transaction<'a> {
    /// Commit and consume the transaction.
    pub fn commit(mut self) -> Result<()> {
        self.inner.commit()
    }

    /// Access the interface to persist data to the store.
    pub fn persist(&mut self) -> Persist {
        let inner = self.inner.persist();
        Persist { inner }
    }

    /// Rollback and consume the transaction.
    pub fn rollback(mut self) -> Result<()> {
        self.inner.rollback()
    }
}
