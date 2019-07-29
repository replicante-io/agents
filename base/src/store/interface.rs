use std::ops::Deref;
use std::sync::Arc;

use crate::Result;

/// Definition of top level store operations.
///
/// See `crate::store::Store` for descriptions of methods.
pub trait StoreInterface: Send + Sync {
    /// Perform database initialisation and applies migrations.
    fn migrate(&self) -> Result<()>;
}

/// Dynamic dispatch all operations to a backend-specific implementation.
#[derive(Clone)]
pub struct StoreImpl(Arc<dyn StoreInterface>);

impl StoreImpl {
    pub fn new<S: StoreInterface + 'static>(store: S) -> StoreImpl {
        StoreImpl(Arc::new(store))
    }
}

impl Deref for StoreImpl {
    type Target = dyn StoreInterface + 'static;
    fn deref(&self) -> &(dyn StoreInterface + 'static) {
        self.0.deref()
    }
}
