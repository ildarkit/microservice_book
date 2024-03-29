use serde_derive::Deserialize;
use config::{Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub database: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("address", "127.0.0.1:8001")?
            .set_default("database", "postgresql://postgres:password@localhost/")?
            .add_source(Environment::with_prefix("USERS"))
            .build()?
            .try_deserialize::<Self>()
    }
}
