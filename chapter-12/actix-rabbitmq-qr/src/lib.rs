pub mod queue_actor;
pub mod state;

use actix::Message;
use lapin::{Connection, ConnectionProperties,
    Channel, Error as LapinError, Queue};
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use serde_derive::{Deserialize, Serialize};
use anyhow::Error;
use lazy_static::lazy_static;
use async_once::AsyncOnce;

pub const REQUESTS: &str = "requests";
pub const RESPONSES: &str = "responses";

lazy_static!{
    static ref CHANNEL: AsyncOnce<Channel> = AsyncOnce::new(
        async {
            let options = ConnectionProperties::default();
            let addr = std::env::var("AMQP_ADDR")
                .unwrap_or_else(|_| "amqp://127.0.0.1:5672".into());
            let conn = Connection::connect(&addr, options).await.unwrap();
            conn.create_channel().await.unwrap()
        }
    );
}

pub async fn get_channel() -> &'static Channel { 
    CHANNEL.get().await
}

pub async fn ensure_queue(chan: &Channel, name: &str)
    -> Result<Queue, LapinError>
{
    let opts = QueueDeclareOptions {
        auto_delete: true,
        ..Default::default()
    };
    let table = FieldTable::default();
    chan.queue_declare(name, opts, table).await
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QrRequest {
    pub image: Vec<u8>,
}

impl Message for QrRequest {
    type Result = ();
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum QrResponse {
    Succeed(String),
    Failed(String),
}

impl From<Result<String, Error>> for QrResponse {
    fn from(res: Result<String, Error>) -> Self {
        match res {
            Ok(data) => QrResponse::Succeed(data),
            Err(err) => QrResponse::Failed(err.to_string()),
        }
    }
}

impl Message for QrResponse {
    type Result = ();
}
