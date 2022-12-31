mod handlers;

use anyhow::Error;
use actix_web::{web, middleware::Logger, App, HttpServer};
use actix_rabbitmq_qr::queue_actor::{QueueActor, QueueHandler, TaskId};
use actix_rabbitmq_qr::{QrRequest, QrResponse, REQUESTS, RESPONSES};
use actix_rabbitmq_qr::state::State;
use actix_rabbitmq_qr::state::tasks::{SharedTasks, init_tasks, Status};

#[derive(Clone)]
pub struct ServerHandler {
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
                rec.set_status(Status::Done(incoming));
            });
        Ok(None)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let addr = "amqp://127.0.0.1:5672";
    let tasks = init_tasks(); 
    let addr = QueueActor::new(
        ServerHandler {
            tasks: tasks.clone(),
        },
        addr,
    ).await.unwrap(); 
 
    HttpServer::new(move || {
        let state = State::new(
            tasks.clone(),
            addr.clone(),
        );
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
    .await?;
    Ok(())
}
