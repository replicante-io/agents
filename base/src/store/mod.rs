use crate::Result;

mod backend;
mod interface;

pub use self::backend::backend_factory;

/// Interface to the agent's persistent storage.
#[derive(Clone)]
pub struct Store {
    inner: self::interface::StoreImpl,
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
        let inner = self::interface::StoreImpl::new(inner);
        Store { inner }
    }

    pub fn with_transaction<F, T>(&self, block: F) -> Result<T>
    where
        F: FnOnce(&Transaction) -> Result<T>
    {
        let tx = Transaction {};
        block(&tx)
    }
}

/// TODO
pub struct Transaction {
    // TODO
}
