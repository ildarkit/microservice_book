use log::debug;
use anyhow::{Error, Context};
use queens_rock::Scanner;
use actix_rabbitmq_qr::queue_actor::{QueueActor, QueueHandler, TaskId};
use actix_rabbitmq_qr::{QrRequest, QrResponse, REQUESTS, RESPONSES};

struct WorkerHandler {}

impl QueueHandler for WorkerHandler {
    type Incoming = QrRequest;
    type Outgoing = QrResponse;

    fn incoming(&self) -> &str {
        REQUESTS
    }

    fn outgoing(&self) -> &str {
        RESPONSES
    }

    fn handle(&self, _: &TaskId, incoming: Self::Incoming)
        -> Result<Option<Self::Outgoing>, Error>
    {
        debug!("In: {:?}", incoming);
        let outgoing = self.scan(&incoming.image).into();
        debug!("Out: {:?}", outgoing);
        Ok(Some(outgoing))
    }
}

impl WorkerHandler {
    fn scan(&self, data: &[u8]) -> anyhow::Result<String> {
        let image = image::load_from_memory(data)
            .context("Worker scan error")?;
        let luma = image.to_luma8().into_vec();
        let scanner = Scanner::new(
            luma.as_ref(),
            image.width() as usize,
            image.height() as usize,
        );
        scanner.scan().extract(0)
            .ok_or_else(|| Error::msg("Can't extract"))
            .and_then(|code| code.decode().map_err(
                |_| Error::msg("Can't decode"))
            )
            .and_then(|data| {
                data.try_string()
                    .map_err(
                        |_| Error::msg("Can't convert to a string")
                    )
            })
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let addr = "amqp://127.0.0.1:5672";
    QueueActor::new(WorkerHandler {}, addr).await?;
    Ok(())
}
