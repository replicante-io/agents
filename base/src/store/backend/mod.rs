use slog::Logger;

use replicante_util_tracing::MaybeTracer;

use crate::config::Agent as Config;
use crate::store::interface::StoreImpl;
use crate::store::Store;
use crate::Result;

#[cfg(any(test, feature = "with_test_support"))]
pub mod mock;
mod sqlite3;

/// Instantiate a new storage backend based on the given configuration.
pub fn backend_factory(config: &Config, logger: Logger, tracer: MaybeTracer) -> Result<Store> {
    let inner = self::sqlite3::Store::new(logger.clone(), config.db.clone(), tracer)?;
    let inner = StoreImpl::new(inner);
    Ok(Store { inner, logger })
}
