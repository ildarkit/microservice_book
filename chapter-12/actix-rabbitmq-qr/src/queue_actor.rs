use super::{ensure_queue, spawn_client};
use actix::fut::wrap_future;
use actix::prelude::*;
use futures::Future;
use lapin::options::{BasicConsumeOptions, BasicPublishOptions};
use lapin::protocol::BasicProperties;
use lapin::{Channel, Error as LapinError};
use lapin::message::Delivery;
use lapin::types::FieldTable;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use uuid::Uuid;
use anyhow::Error;

pub type TaskId = String;

pub trait QueueHandler: 'static {
    type Incoming: for<'de> Deserialize<'de>;
    type Outgoing: Serialize;

    fn incoming(&self) -> &str;
    fn outgoing(&self) -> &str;
    fn handle(
        &self,
        id: &TaskId,
        incoming: Self::Incoming,
    ) -> Result<Option<Self::Outgoing>, Error>;
}

pub struct QueueActor<T: QueueHandler> {
    channel: Channel<TcpStream>,
    handler: T,
}

impl<T: QueueHandler> Actor for QueueActor<T> {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}
}

impl<T: QueueHandler> QueueActor<T> {

    pub fn new(handler: T, mut sys: &mut SystemRunner)
        -> Result<Addr<Self>, Error>
    {
        let channel = spawn_client(&mut sys)?;
        let chan = channel.clone();
        let fut = ensure_queue(&chan, handler.outgoing());
        sys.block_on(fut)?;
        let fut = ensure_queue(&chan, handler.incoming())
            .and_then(move |queue| {
                let opts = BasicConsumeOptions {
                    ..Default::default()
                };
                let table = FieldTable::new();
                let name = format!("{}-consumer", queue.name());
                chan.basic_consume(&queue, &name, opts, table)
            });
        let stream = sys.block_on(fut)?;
        let addr = QueueActor::create(move |ctx| {
            ctx.add_stream(stream);
            Self { channel, handler}
        });
        Ok(addr)
    }
}

impl<T: QueueHandler> StreamHandler<Delivery, LapinError> for QueueActor<T> {

    fn handler(&mut self, item: Delivery, ctx: &mut Context<Self>) {
        debug!("Message received!");
        let fut = self.channel
            .basic_ack(item.delivery_tag, false)
            .map_err(drop);
        ctx.spawn(wrap_future(fut));
        match self.process_message(item, ctx) {
            Ok(pair) => {
                if let Some((corr_id, data)) = pair {
                    self.send_message(corr_id, data, ctx);
                }
            },
            Err(err) => {
                warn!("Message processing error: {}", err);
            },
        }
    }
}

