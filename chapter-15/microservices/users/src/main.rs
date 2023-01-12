mod handlers;
mod settings;

use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use rouille::Response;
use handlers::handler;
use settings::Settings;

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let config = Settings::new()?;
    let manager = ConnectionManager::<PgConnection>::new(&config.database);
    let pool = r2d2::Pool::new(manager)?;

    rouille::start_server(config.address, move |request| {
        match handler(&request, &pool) {
            Ok(response) => response,
            Err(err) => {
                Response::text(err.to_string())
                    .with_status_code(500)
            }
        }
    })
}
