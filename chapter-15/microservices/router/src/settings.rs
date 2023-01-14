use serde_derive::Deserialize;
use config::{Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub signup: String,
    pub signin: String,
    pub new_comment: String,
    pub comments: String,
    pub radis: String,
    pub address: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("address", "127.0.0.1:8080")?
            .set_default("signup", "http://127.0.0.1:8001/signup")?
            .set_default("signin", "http://127.0.0.1:8001/signin")?
            .set_default("new_comment", "http://127.0.0.1:8003/new_comment")?
            .set_default("comments", "http://127.0.0.1:8003/comments")?
            .set_default("radis", "redis://127.0.0.1:6379")?
            .add_source(Environment::with_prefix("ROUTER"))
            .build()?
            .try_deserialize::<Self>()
    }
}
