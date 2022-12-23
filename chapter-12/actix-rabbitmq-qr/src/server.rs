use actix::{Addr, System};
use actix_web::http::{self, header, StatusCode};
use actix_web::{rt, middleware, server, App, Error as WebError,
    HttpMessage, HttpRequest, HttpResponse, HttpServer};
use askama::Template;
use chrono::{DateTime, Utc};
use futures::{future, Future, Stream};
use indexmap::IndexMap;
use log::debug;
use actix_rabbitmq_qr::queue_actor::{QueueActor, QueueHandler, SendMessage, TaskId};
use actix_rabbitmq_qr::{QrRequest, QrResponse, REQUESTS, RESPONSES};
use actix_rabbitmq_qr::handlers;
use std::fmt;
use std::sync::{Arc, Mutex};

type SharedTasks = Arc<Mutex<IndexMap<String, Record>>>;

#[derive(Clone)]
struct Record {
    task_id: TaskId,
    timestamp: DateTime<Utc>,
status: Status,
}

#[derive(Clone)]
enum Status {
    InProgress,
    Done(QrResponse),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::InProgress => write!(f, "in progress"),
            Status::Done(resp) => match resp {
                QrResponse::Succeed(data) => write!(f, "done: {data}"),
                QrResponse::Failed(err) => write!(f, "failed: {err}"),
            },
        }
    }
}

#[derive(Clone)]
struct State {
    tasks: SharedTasks,
    addr: Addr<QueueActor<ServerHandler>>,
}

struct ServerHandler {
    tasks: SharedTasks,
}

impl QueueHandler for ServerHandler {
    type Incoming = QrResponse;
    type Outgoing = QrRequest;

    fn incoming(&self) -> &str {
        RESPONSES
    }

    fn outgoing(&self) -> &str {
        REQUESTS
    }

    fn handle(&self, id: &TaskId, incoming: Self::Incoming)
        -> Result<Option<Self::Outgoing>, Error>
    {
        self.tasks.lock().unwrap().get_mut(id)
            .map(move |rec| {
                rec.status = Status::Done(incoming);
            });
        Ok(None)
    }
}

async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut sys = rt::System::new();
    let tasks = Arc::new(Mutex::new(IndexMap::new()));
    let addr = QueueActor::new(
        ServerHandler {
            tasks: tasks.clone(),
        },
        &mut sys,
    )?;

    let state = State {
        tasks: tasks.clone(),
        addr,
    };

    sys.block_on(
        HttpServer::new(move || { 
            let data = web::Data::new(state);
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::clone(&data))
                .route("/", web::get().to(handlers::index))
                .route("/task", web::post().to(handlers::upload))
                .route("/tasks", web::get().to(handlers::tasks)) 
        })
        .bind(("127.0.0.1", 8080))?
        .run()
    )
}
