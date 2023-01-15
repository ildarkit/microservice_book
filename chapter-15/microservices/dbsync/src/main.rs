#[macro_use] extern crate diesel_migrations;

use diesel::prelude::*;
use anyhow::{Error, Result};
use log::debug;
use serde_derive::Deserialize;
use config::{Config, ConfigError, Environment};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug, Deserialize, Clone)]
struct Settings {
    database: String,
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("database", "postgres://localhost/")?
            .add_source(Environment::with_prefix("DBSYNC"))
            .build()?
            .try_deserialize::<Self>()
    }
}

fn migrate(conn: &mut PgConnection)
    -> Result<()>
{
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|_| Error::msg("Failed to migrate database"))?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or("info")
    );
    let config = Settings::new()?;
    debug!("Waiting for database...");
    loop {
        let conn: Result<PgConnection, _> =
            Connection::establish(&config.database);
        if let Ok(mut conn) = conn {
            debug!("Database connected");
            migrate(&mut conn)?;
            break;
        }
    }
    debug!("Database migrated");
    Ok(())
}
