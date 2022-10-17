use std::fmt;
use std::sync::{Arc, Mutex};
use slab::Slab;
use futures::{future, Future};
use hyper::{Body, Response, Server, Error, Method, Request, StatusCode};
use hyper::service::service_fn;

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| microservice_handler(req, &user_db))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}

fn microservice_handler(req: Request<Body>, user_db: &UserDb) 
    -> impl Future<Item=Response<Body>, Error=Error>
{
    let response = {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                Response::new(INDEX.into())
            },
            (method, path) if path.starts_with(USER_PATH) => {
                unimplemented!();
            },
            _ => {
                response_with_code(StatusCode::NOT_FOUND)
            },
        }
    };
    future::ok(response)
}

fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
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
