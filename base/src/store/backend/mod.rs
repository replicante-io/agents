use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use crate::config::Agent as Config;
use crate::store::interface::StoreImpl;
use crate::store::Store;
use crate::Result;

#[cfg(any(test, feature = "with_test_support"))]
pub mod mock;
mod sqlite3;

/// Instantiate a new storage backend based on the given configuration.
pub fn backend_factory<T>(config: &Config, logger: Logger, _tracer: T) -> Result<Store>
where
    T: Into<Option<Arc<Tracer>>>,
{
    let inner = self::sqlite3::Store::new(logger, config.db.clone())?;
    let inner = StoreImpl::new(inner);
    Ok(Store { inner })
}
