use serde_derive::Deserialize;
use config::{Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub smtp_address: String,
    pub smtp_login: String,
    pub smtp_password: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("address", "0.0.0.0:8000")?
            .set_default("smtp_address", "127.0.0.1:2525")?
            .add_source(Environment::with_prefix("MAILS"))
            .build()?
            .try_deserialize::<Self>()
    }
}
