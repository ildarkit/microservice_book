use std::sync::Arc;
use failure::Error;
use gotham::handler::HandlerResult;
use gotham::router::Router;
use gotham::router::build_simple_router;
use gotham::state::{FromState, State};
use gotham::mime::TEXT_HTML_UTF_8;
use gotham::helpers::http::response::create_response;
use gotham::prelude::*;
use hyper::StatusCode;
use hyper::header::{HeaderMap, USER_AGENT};
use tokio::runtime::Runtime;
use tokio_postgres::NoTls;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::sync::Mutex;
use tracing::{instrument, trace, info, error};
use tracing_subscriber;
use tracing_subscriber::prelude::*;
use console_subscriber;
#[macro_use]
extern crate lazy_static;

struct StatusChannel {
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl StatusChannel {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tx,
            rx
        }
    }

    fn get_sender(&self) -> &Sender<String> {
        &self.tx
    }

    fn get_receiver(&mut self) -> &mut Receiver<String> {
        &mut self.rx
    }
}

lazy_static! {
    static ref STATUS_CHANNEL: Arc<Mutex<StatusChannel>> = {
        let s = Arc::new(Mutex::new(StatusChannel::new()));
        s
    };
}

#[instrument]
fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/")
            .to_async(register_user_agent);
    })
} 

#[instrument(skip(state))]
async fn register_user_agent(state: State) -> HandlerResult {
    let user_agent = HeaderMap::borrow_from(&state)
        .get(USER_AGENT)
        .map(|value| value.to_str().unwrap())
        .unwrap_or_else(|| "<undefined>");

    let mutex_sender = STATUS_CHANNEL.lock().await;
    let sender = mutex_sender.get_sender();
    trace!("Sending to channel");
    let (status, body) = match sender.send(user_agent.to_string()).await {
        Ok(_) => {
            trace!("Sended successfully");
            (StatusCode::OK,
            format!("User-Agent: {}",user_agent))
        }
        Err(err) => {
            error!("Channel sending fail");
            (StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string())
        }
    };
    trace!("Responding");
    let res = create_response(
        &state,
        status,
        TEXT_HTML_UTF_8,
        body);
    trace!("Return response");
    Ok((state, res))
}

fn main() -> Result<(), Error> {
    let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        .with(console_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rt = Runtime::new().unwrap();
    let handshake =
        tokio_postgres::connect("postgres://postgres@localhost:5432", NoTls);
    let (client, connection) = rt.block_on(handshake)?;
    rt.spawn(async move {
        match connection.await {
            Ok(res) => {
                trace!("Connected to database");
                Ok(res)
            }
            Err(e) => {
                error!("Connection error: {}", e);
                Err(e)
            }
        }
    });
    rt.block_on(async {
        client.batch_execute(
        "CREATE TABLE IF NOT EXISTS agents (
            agent TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );").await.unwrap();
        trace!("Table created");
    });
    rt.spawn(async move {
        let mut rx_mutex = STATUS_CHANNEL.lock().await;
        let rx = rx_mutex.get_receiver();
        while let Some(user_agent) = rx.recv().await {
            client.query("INSERT INTO agents (agent) VALUES ($1) RETURNING agent", 
                         &[&user_agent])
                .await.unwrap();
            trace!("Data was saved in database");
        }
    });
    let addr = "127.0.0.1:7878";
    info!("Listening for requests at http://{}", addr);
    gotham::start(addr, router()).unwrap();

    Ok(())
}
