use std::sync::Once;
use std::borrow::BorrowMut;
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
use tokio::sync::{Mutex, RwLock};
use tracing::{instrument, trace, info, error};
use tracing_subscriber;
use tracing_subscriber::prelude::*;
use console_subscriber;

static mut SENDER: Option<RwLock<Sender<String>>> = None;
static mut RECEIVER: Option<Box<Mutex<Receiver<String>>>> = None;
static INIT: Once = Once::new();

fn init_channel(size: usize) {
    INIT.call_once(|| {
        let (tx, rx) = mpsc::channel(size);
        unsafe {
            *SENDER.borrow_mut() = Some(RwLock::new(tx));
            *RECEIVER.borrow_mut() = Some(Box::new(Mutex::new(rx)));
        }
    });
}

fn get_sender<'a>() -> &'a RwLock<Sender<String>> {
    unsafe { SENDER.as_ref().unwrap() }
}

fn get_receiver<'a>() -> &'a mut Mutex<Receiver<String>> {
    unsafe { RECEIVER.as_mut().unwrap() }
}

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

    let sender = get_sender().read().await;
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
    init_channel(100);

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
        if let Err(e) = connection.await {
            error!("Connection error: {}", e);
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
        let mut rx = get_receiver().lock().await;
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
