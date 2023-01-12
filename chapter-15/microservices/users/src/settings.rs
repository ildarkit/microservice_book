use serde_derive::Deserialize;
use config::{File, Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub database: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("address", "0.0.0.0:8000")?
            .set_default("database", "postgres://localhost/")?
            .add_source(File::with_name("config"))
            .add_source(Environment::with_prefix("USERS"))
            .build()?
            .try_deserialize::<Self>()
    }
}
