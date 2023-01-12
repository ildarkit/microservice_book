mod handlers;

use std::env;
use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use rouille::Response;
use dotenvy::dotenv;
use handlers::handler;

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or("test.db".to_string());
    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
    let pool = r2d2::Pool::new(manager)?;

    rouille::start_server("127.0.0.1:8001", move |request| {
        match handler(&request, &pool) {
            Ok(response) => response,
            Err(err) => {
                Response::text(err.to_string())
                    .with_status_code(500)
            }
        }
    })
}
