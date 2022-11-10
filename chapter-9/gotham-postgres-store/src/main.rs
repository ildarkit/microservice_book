use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use failure::Error;
use gotham::handler::HandlerResult;//{HandlerError, HandlerResult};
//use gotham::middleware::state::StateMiddleware;
//use gotham::pipeline::single_pipeline;
//use gotham::pipeline::single_middleware;
use gotham::router::Router;
use gotham::router::build_simple_router;//{DefineSingleRoute, DrawRoutes, build_router};
use gotham::state::{FromState, State};
use gotham::mime::TEXT_HTML_UTF_8;
use gotham::helpers::http::response::create_response;
use gotham::prelude::*;
use hyper::StatusCode;//{Body, Response, StatusCode};
use hyper::header::{HeaderMap, USER_AGENT};
use tokio::task;
use tokio_postgres::NoTls;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::sync::Mutex;
#[macro_use]
extern crate lazy_static;

struct StatusChannel {
    tx: Option<Sender<String>>,
    rx: Option<Receiver<String>>,
}

impl Deref for StatusChannel {
    type Target = Option<Receiver<String>>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl DerefMut for StatusChannel {
    //type Target = Option<Receiver<String>>;

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

impl StatusChannel {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tx: Some(tx),
            rx: Some(rx)
        }
    }

    fn get_sender(&self) -> Option<&Sender<String>> {
        self.tx.as_ref()
    }

    fn get_receiver(&mut self) -> Option<&mut Receiver<String>> {
        self.rx.as_mut()
    }
}

lazy_static! {
    static ref STATUS_CHANNEL: Arc<Mutex<StatusChannel>> = {
        let s = Arc::new(Mutex::new(StatusChannel::new()));
        s
    };
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/")
            .to_async(register_user_agent);
    })
} 

async fn register_user_agent(state: State) -> HandlerResult {
    let user_agent = HeaderMap::borrow_from(&state)
        .get(USER_AGENT)
        .map(|value| value.to_str().unwrap())
        .unwrap_or_else(|| "<undefined>");

    let mutex_sender = STATUS_CHANNEL.lock().await;
    let sender = mutex_sender.get_sender().unwrap();
    let (status, body) = match sender.send(user_agent.to_string()).await {
        Ok(_) => {
            (StatusCode::OK,
            format!("User-Agent: {}",user_agent))
        }
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string())
        }
    };

    let res = create_response(
        &state,
        status,
        TEXT_HTML_UTF_8,
        body);
    Ok((state, res))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect("postgres://postgres@localhost:5432", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let execute = client.batch_execute(
        "CREATE TABLE IF NOT EXISTS agents (
            agent TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );");
    task::spawn_blocking(move || execute).await?;

    let mut rx_mutex = STATUS_CHANNEL.lock().await;
    let rx = &mut rx_mutex.get_receiver().unwrap();
    
    tokio::spawn(async {
        while let Some(user_agent) = rx.recv().await {
            client.query("INSERT INTO agents (agent) VALUES ($1) RETURNING agent", 
                         &[&user_agent])
                .await.unwrap();
        }
    });

    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router()).unwrap();

    Ok(())
}
