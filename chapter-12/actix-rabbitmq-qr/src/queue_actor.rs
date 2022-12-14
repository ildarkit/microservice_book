use super::{ensure_queue, get_channel};
use actix::fut::wrap_future;
use actix::prelude::*;
use lapin::options::{
    BasicConsumeOptions,
    BasicPublishOptions,
    BasicAckOptions};
use lapin::protocol::BasicProperties;
use lapin::{Error as LapinError, Channel};
use lapin::message::Delivery;
use lapin::types::FieldTable;
use log::{debug, warn, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Error, Context as AnyhowContext};
use amq_protocol_types::ShortString;
use thiserror;
use serde_json;

pub type TaskId = String;

pub trait QueueHandler: Unpin + 'static {
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

#[derive(thiserror::Error, Debug)]
pub enum QueueActorError {
    #[error("Message has no address for the response")]
    NoAddressError,
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    UnexpectedError(#[from] Error),
}

pub struct QueueActor<T: QueueHandler> {
    handler: T,
    channel: Channel,
}

impl<T: QueueHandler> Actor for QueueActor<T> {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}
}

impl<T: QueueHandler> QueueActor<T> {

    pub async fn new(handler: T, addr: &str)
        -> Result<Addr<Self>, Error>
    {
        let channel = get_channel(addr).await?;
        let chan = channel.clone();
        ensure_queue(&chan, handler.outgoing()).await?;

        let queue = ensure_queue(&chan, handler.incoming()).await?;
        let opts = BasicConsumeOptions {
            ..Default::default()
        };
        let table = FieldTable::default();
        let tag = format!("{}-consumer", queue.name());
        
        let stream = chan.basic_consume(
            &queue.name().as_str(),
            &tag,
            opts,
            table
        ).await?;

        let addr = QueueActor::create(move |ctx| {
            ctx.add_stream(stream);
            Self { handler, channel }
        });
        Ok(addr)
    }
}

impl<T: QueueHandler> StreamHandler<Result<Delivery, LapinError>> for QueueActor<T> {

    fn handle(&mut self,
        item: Result<Delivery, LapinError>,
        ctx: &mut Context<Self>
    )
    { 
        match item {
            Ok(item) => {
                debug!("Message received!");
                match self.process_message(&item, ctx) {
                    Ok(pair) => {
                        if let Some((corr_id, data)) = pair {
                            self.send_message(corr_id, data, ctx);
                        }
                    },
                    Err(err) => {
                        warn!("Message processing error: {}", err);
                    },
                }
                let fut = async move {
                    item.ack(
                        BasicAckOptions::default()
                    )
                    .await
                    .unwrap_or_else(|_| ())
                };
                ctx.spawn(wrap_future(fut)); 
            },
            Err(e) => error!("Message is not received:\n\t {e}"),
        }
    }
}

pub struct SendMessage<T>(pub T);

impl<T> Message for SendMessage<T> {
    type Result = TaskId;
}

impl<T: QueueHandler> Handler<SendMessage<T::Outgoing>> for QueueActor<T> {
    type Result = TaskId;

    fn handle(&mut self, msg: SendMessage<T::Outgoing>, ctx: &mut Self::Context)
        -> Self::Result
    {
        let corr_id = Uuid::new_v4().simple().to_string();
        self.send_message(corr_id.clone(), msg.0, ctx);
        corr_id
    }
}

impl<T: QueueHandler> QueueActor<T> {

    fn process_message(&self, item: &Delivery, _: &mut Context<Self>)
        -> Result<Option<(String, T::Outgoing)>, QueueActorError>
    {
        let corr_id = item.properties.correlation_id()
            .to_owned()
            .ok_or_else(|| QueueActorError::NoAddressError)?;
        let incoming = serde_json::from_slice(&item.data)
            .context("Deserializing message data error")?;
        let outgoing = self.handler.handle(&corr_id.to_string(), incoming)?;
        if let Some(outgoing) = outgoing {
            Ok(Some((corr_id.to_string(), outgoing)))
        } else {
            Ok(None)
        }
    }

    fn send_message(
        &self,
        corr_id: String,
        outgoing: T::Outgoing,
        ctx: &mut Context<Self>)
    {
        let data = serde_json::to_vec(&outgoing);
        match data {
            Ok(data) => {
                let opts = BasicPublishOptions::default();
                let props = BasicProperties::default()
                    .with_correlation_id(ShortString::from(corr_id));
                debug!("Sending to: {}", self.handler.outgoing());
                let outgoing = self.handler.outgoing().to_string();
                let channel = self.channel.clone();
                let fut = async move {
                    channel
                        .basic_publish("", &outgoing, opts, &data, props)
                        .await
                        .map_err(drop)
                        .map(drop)
                        .unwrap();
                };
                ctx.spawn(wrap_future(fut));
            },
            Err(err) => {
                error!("Can't encode an outgoing message: {}", err);
            },
        }
    }
}
