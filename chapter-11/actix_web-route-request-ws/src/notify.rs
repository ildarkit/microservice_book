use log::error;
use actix::{
    Actor,
    ActorContext,
    AsyncContext,
    Handler,
    Recipient,
    StreamHandler
};
use actix_web_actors::ws;
use crate::repeater::{RepeaterControl, RepeaterUpdate};
use std::time::{Duration, Instant};

const PING_INTERVAL: Duration = Duration::from_secs(20);
const PING_TIMEOUT: Duration = Duration::from_secs(60);

pub struct NotifyActor {
    last_ping: Instant,
    repeater: Recipient<RepeaterControl>,
}

impl NotifyActor {
    pub fn new(repeater: Recipient<RepeaterControl>) -> Self {
        Self {
            last_ping: Instant::now(),
            repeater,
        }
    }
}

impl Actor for NotifyActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let msg = RepeaterControl::Subscribe(ctx.address().recipient());
        self.repeater.do_send(msg);
        ctx.run_interval(PING_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_ping) > PING_TIMEOUT {
                ctx.stop();
                return;
            }
            ctx.ping(b"ping");
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let msg = RepeaterControl::Unsubscribe(ctx.address().recipient());
        self.repeater.do_send(msg);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotifyActor {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context) 
    {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_ping = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_ping = Instant::now();
            }
            Ok(ws::Message::Text(_)) => (),
            Ok(ws::Message::Binary(_)) => (),
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Err(e) => error!("{e}"),
        }
    }
}

impl Handler<RepeaterUpdate> for NotifyActor {
    type Result = ();

    fn handle(&mut self, msg: RepeaterUpdate, ctx: &mut Self::Context)
        -> Self::Result
    {
        let RepeaterUpdate(comment) = msg;
        if let Ok(data) = serde_json::to_string(&comment) {
            ctx.text(data);
        }
    }
}


