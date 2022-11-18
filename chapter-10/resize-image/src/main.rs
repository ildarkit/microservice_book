use std::thread;
use std::io::Cursor;
use std::io::{Error, ErrorKind};
use serde_json::Value;
use futures::StreamExt;
use hyper::{self, Server, Body, Response, Method, Request, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use futures::{TryFutureExt, FutureExt};
use tokio::sync::{mpsc, oneshot};
use image::ImageResult;
use image::imageops::FilterType;

const INDEX: &'static str = r#"
<!doctype html>
<html>
    <head>
        <title>Rust Image Resize Microservice</title>
    </head>
    <body>
        <h3>Rust Image Resize Microservice</h3>
    </body>
</html>
"#;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type WorkerResponse = Result<Vec<u8>>;

#[derive(Debug)]
struct WorkerRequest {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
    tx: oneshot::Sender<WorkerResponse>,
}

fn start_worker() -> mpsc::Sender<Option<WorkerRequest>> {
    let (tx, mut rx) = mpsc::channel::<Option<WorkerRequest>>(1);
    thread::spawn(move || {
        for req in rx.blocking_recv() {
            if let Some(req) = req {
                let resp = convert(req.buffer, req.width, req.height)
                    .map_err(other);
                req.tx.send(resp).ok();
            }
        }
    });
    tx
}

fn convert(data: Vec<u8>, width: u16, height: u16) -> ImageResult<Vec<u8>> {
    let format = image::guess_format(&data)?;
    let img = image::load_from_memory(&data)?;
    let scaled = img.resize(width as u32, height as u32, FilterType::Lanczos3);
    let mut result = Vec::new();
    scaled.write_to(&mut Cursor::new(&mut result), format)?;
    Ok(result)
}

async fn microservice_handler(tx: mpsc::Sender<Option<WorkerRequest>>, req: Request<Body>)
    -> Result<Response<Body>>
{
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => {
            Ok(Response::new(INDEX.into()))
        },
        (&Method::POST, "/resize") => {
            let (width, height) = {
                let uri = req.uri().query().unwrap_or("");
                let query = queryst::parse(uri).unwrap_or(Value::Null);
                let w = to_number(&query["width"], 180);
                let h = to_number(&query["height"], 180);
                (w, h)
            };
            let response = req.into_body()
                .map(|buf| buf.unwrap().to_vec())
                .concat()
                .then(|buffer| async move {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let resp_rx = resp_rx.map_err(other);
                    let request = WorkerRequest{
                        buffer, width, height, tx: resp_tx };
                    tx.send(Some(request))
                        .map_err(other)
                        .and_then(move |_| resp_rx)
                        .await
                })
                .await?
                .map(|resp| Response::new(resp.into()));
            Ok(response?)
        },
        _ => {
            response_with_code(StatusCode::NOT_FOUND)
        },
    }
}

fn to_number(value: &Value, default: u16) -> u16 {
    value.as_str()
        .and_then(|x| x.parse::<u16>().ok())
        .unwrap_or(default)
}

fn other<E>(err: E) -> GenericError
where 
    E: Into<GenericError>,
{
    Box::new(Error::new(ErrorKind::Other, err))
}

fn response_with_code(status_code: StatusCode) 
    -> Result<Response<Body>>
{
    let resp = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<()> {
    let tx = start_worker();

    let make_service = make_service_fn(|_| {
        let tx = tx.clone();
        async {
            Ok::<_, GenericError>(service_fn(move |req| {
                microservice_handler(tx.clone(), req)
            }))
        }
    });
    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_service);
    server.await?;
    Ok(())
}
