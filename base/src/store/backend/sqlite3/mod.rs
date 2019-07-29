use failure::ResultExt;
use failure::SyncFailure;
use migrant_lib::Config;
use migrant_lib::Migrator;
use migrant_lib::Settings;
use slog::debug;
use slog::info;
use slog::Logger;

use crate::store::interface::StoreInterface;
use crate::ErrorKind;
use crate::Result;

/// SQLite3 backed store.
pub struct Store {
    logger: Logger,
    path: String,
}

impl Store {
    pub fn new(logger: Logger, path: String) -> Result<Store> {
        Ok(Store { logger, path })
    }
}

impl StoreInterface for Store {
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
        config.setup()
            .map_err(SyncFailure::new)
            .with_context(|_| ErrorKind::PersistentMigrate)?;
        config.use_cli_compatible_tags(true);

        // Register migrations.
        macro_rules! make_migration {
            ($tag:expr) => {
                migrant_lib::EmbeddedMigration::with_tag($tag)
                    .up(include_str!(concat!(
                        "./migrations/",
                        $tag,
                        "/up.sql"
                    )))
                    .down(include_str!(concat!(
                        "./migrations/",
                        $tag,
                        "/down.sql"
                    )))
                    .boxed()
            };
        }
        config.use_migrations(&[
            make_migration!("20190728220141_initialise"),
        ])
        .map_err(SyncFailure::new)
        .with_context(|_| ErrorKind::PersistentMigrate)?;

        info!(self.logger, "Running DB migrations as needed");
        let config = config.reload()
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
