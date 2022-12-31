use std::pin::Pin;
use std::error::Error;
use log::debug;
use chrono::Utc;
use askama::Template;
use anyhow::Context;
use futures::{Stream, stream::StreamExt};
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{http::header, web, HttpResponse, error::ResponseError};
use serde_derive::Serialize;
use actix_rabbitmq_qr::QrRequest;
use actix_rabbitmq_qr::state::{State, tasks::{Record, Status}};
use actix_rabbitmq_qr::queue_actor::SendMessage;
use crate::ServerHandler;

#[derive(thiserror::Error, Debug)]
pub enum WebError {
    #[error("Can't run upload image task")]
    TaskError,
    #[error(transparent)]
    MultipartError(#[from] MultipartError),
    #[error(transparent)]
    TemplateError(#[from] askama::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl WebError {
    fn get_cause(&self) -> String {
        match self.source() {
            Some(s) => s.to_string(),
            None => "Unknown".into()
        }
    } 
}

impl ResponseError for WebError {

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(WebErrorResponse {
                error: self.to_string(),
                cause: self.get_cause(),
            })
    } 
}

#[derive(Serialize)]
struct WebErrorResponse {
    error: String,
    cause: String,
}

#[derive(Template)]
#[template(path = "tasks.html")]
struct Tasks {
    tasks: Vec<Record>,
}

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("QR Parsing Microservice")
}

pub async fn tasks(state_tasks: web::Data<State<ServerHandler>>)
    -> Result<HttpResponse, WebError>
{
    let tasks = state_tasks.tasks.lock()
        .map_err(|_| anyhow::Error::msg("Can't get tasks"))?
        .values().cloned().collect();
    let tmpl = Tasks{ tasks };
    let body = tmpl.render().context("Can't render tasks")?;
    Ok(HttpResponse::Ok().body(body))
}

pub async fn upload(
    state_tasks: web::Data<State<ServerHandler>>,
    payload: Multipart
)
    -> Result<HttpResponse, WebError>
{
    let (bytes, _) = payload
        .map(handle_multipart_item)
        .flatten()
        .into_future()
        .await;
    let bytes = match bytes {
        Some(bytes) => Ok(bytes),
        None => Err(MultipartError::Incomplete),
    };
    let image = bytes.context("Unexpected payload error")?;
    debug!("Image: {:?}", image);

    let request = QrRequest { image };
    let task_id = state_tasks
        .addr.send(SendMessage(request))
        .await.map_err(|_| WebError::TaskError)?;

    let record = Record {
        task_id: task_id.clone(),
        timestamp: Utc::now(),
        status: Status::InProgress,
    };
    state_tasks.tasks.lock()
        .map_err(|_| anyhow::Error::msg("Can't insert upload task"))?
        .insert(task_id, record);
            
    Ok(redirect("/tasks"))
}

fn handle_multipart_item(item: Result<Field, MultipartError>)
    -> Pin<Box<dyn Stream<Item = Vec<u8>>>>
{
    let field = item.unwrap();
    Box::pin(
        field.map(|bytes| bytes.unwrap().to_vec())
    )
}

fn redirect(url: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, url))
        .finish()
}
