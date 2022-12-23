use actix_web::error::MultipartError;
use actix_multipart::Multipart;
use actix_web::{web, Error as WebError, HttpRequest, HttpResponse};
use actix_rabbitmq_qr::state::tasks::Record;

#[derive(Template)]
#[template(path = "tasks.html")]
struct Tasks {
    tasks: Vec<Record>,
}

fn index(_: &HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("QR Parsing Microservice")
}

fn task(_req: HttpRequest, tasks: web::Data<>) -> impl Future<Output = HttpResponse> {
    let tasks: Vec<_> = req
        .state()
        .tasks
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect();
    let tmpl = Tasks{tasks};
    future::ok(HttpResponse::Ok().body(tmpl.render().unwrap()))
}

fn upload(req: HttpRequest<State>) -> impl Future<Output = HttpResponse> {
    req.multipart()
        .map(handle_multipart_item)
        .flatten()
        .into_future()
        .and_then(|(bytes, stream)| {
            if let Some(bytes) = bytes {
                Ok(bytes)
            } else {
                Err((MultipartError::Incomplete, stream))
            }
        })
        .map_err(|(err, _)| WebError::from(err))
        .and_then(move |image| {
            debug!("Image: {:?}", image);
            let request = QrRequest { image };
            req.state()
                .addr.send(SendMessage(request))
                .from_err()
                .map(move |task_id| {
                    let record = Record {
                        task_id: task_id.clone(),
                        timestamp: Utc::now(),
                        status: Status::InProgress,
                    };
                    req.state().tasks.lock().unwrap().insert(task_id, record);
                    req
                })
        })
        .map(|req| {
            HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/tasks")
                .finish()
        })
}

pub fn handle_multipart_item(item: MultipartItem<Payload>)
    -> Box<Stream<Item = Vec<u8>, Error = MultipartError>>
{
    match item {
        MultipartItem::Field(field) => {
            Box::new(
                field.concat2()
                    .map(|bytes| bytes.to_vec())
                    .into_stream()
            )
        },
        MultipartItem::Nested(mp) => {
            Box::new(mp.map(handle_multipart_item).flatten())
        },
    }
}
