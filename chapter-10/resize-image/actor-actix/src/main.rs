use std::thread;
use std::io::{Error, ErrorKind};
use failure::Fail;
use serde_json::Value;
use futures::{Future, StreamExt, FutureExt, TryFutureExt};
use hyper::{self, Server, Body, Response, Method, Request, StatusCode};
use hyper::service::service_fn;
use tracing::info;
use tracing_subscriber;
use actix::{Actor, Addr};
use actix::sync::SyncArbiter;
use tower::make::Shared;

mod actors;

use self::actors::{
    count::{Count, CountActor},
    log::{Log, LogActor},
    resize::{Resize, ResizeActor},
};

#[derive(Clone)]
struct State {
    resize: Addr<ResizeActor>,
    count: Addr<CountActor>,
    log: Addr<LogActor>,
}

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

fn count_up(state: &State, path: &str) -> impl Future<Output=Result<()>> {
   let path = path.to_string();
   let log = state.log.clone();
   state.count.send(Count(path.clone()))
       .and_then(move |value| {
           let message = format!("total requests for '{}' is {}", path , value);
           log.send(Log(message))
       })
       .map_err(|err| other(err.compat()))
}

async fn microservice_handler(state: &State, req: Request<Body>)
    -> Result<Response<Body>>
{
    info!("Handling a new request {:?}", req);
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => {
            count_up(state, "/").await?;
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
            info!(width = %width, height = %height);
            let resize = state.resize.clone();
            let response = req.into_body()
                .map(|buf| buf.unwrap().to_vec())
                .concat()
                .then(|buffer| async {
                    info!(request_buffer_length = %buffer.len()); 
                    let msg = Resize {
                        buffer,
                        width,
                        height
                    };
                    resize.send(msg)
                        .map_err(|err| other(err.compat()))
                        .await
                })
                .await?
                .map(|resp| Response::new(resp.into()));
            count_up(state, "/resize").await?;
            info!("Response resized image");
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

#[actix::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_target(false)
    .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cpu_core_count = thread::available_parallelism()?.get();
    info!(cpu_core_count = %cpu_core_count);

    let resize = SyncArbiter::start(cpu_core_count, || ResizeActor);
    let count = CountActor::new().start();
    let log = LogActor::new().start();
    let state = State { resize, count, log };

    let make_service = Shared::new(
        service_fn(move |req| {
            let state = state.clone();
            async move {
                microservice_handler(&state, req).await
            }
        })
    );

    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_service); 
    info!("Server listening address {}", addr);
    server.await?; 

    Ok(())
}
