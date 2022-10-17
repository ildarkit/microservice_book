use std::fmt;
use std::sync::{Arc, Mutex};
use slab::Slab;
use futures::{future, Future};
use hyper::{Body, Response, Server, Error, Method, Request, StatusCode};
use hyper::service::service_fn;

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| service_fn(microservice_handler));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}

fn microservice_handler(req: Request<Body>) 
    -> impl Future<Item=Response<Body>, Error=Error>
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            future::ok(Response::new(INDEX.into()))
        },
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            future::ok(response)
        },
    }
}

const INDEX: &'static str = r#"
<!doctype html>
<html>
    <head>
        <title>Rust Microservice</title>
    </head>
    <body>
        <h3>Rust Microservice</h3>
    </body>
</html>
"#;

type UserId = u64;
struct UserData;
type UserDb = Arc<Mutex<Slab<UserData>>>;   
