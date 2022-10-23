extern crate futures;
extern crate hyper;
extern crate rand;
extern crate tokio;

use hyper::{Body, Response, Server, Method, Request, StatusCode};
use hyper::service::service_fn;
use futures::{future, Stream, Future};
use std::path::Path;
use std::fs;
use tokio::fs::File;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use std::io::{Error, ErrorKind};


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



fn microservice_handler(req: Request<Body>, files: &Path)
    -> Box<dyn Future<Item=Response<Body>, Error=std::io::Error> + Send> 
{
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/upload") => {
            let name: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .collect();
            let mut filepath = files.to_path_buf();
            filepath.push(&name);
            let create_file = File::create(filepath);
            let write = create_file.and_then(|file| {
                req.into_body()
                    .map_err(other)
                    .fold(file, |file, chunk| {
                        tokio::io::write_all(file, chunk)
                            .map(|(file, _)| file)
                    })
            });
            let body = write.map(|_| {
                Response::new(name.into())
            });
            Box::new(body)
        },
        _ => {
            response_with_code(StatusCode::NOT_FOUND)
        },
    }
}


fn other<E>(err: E) -> Error
where 
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, err)
}


fn response_with_code(status_code: StatusCode) 
    -> Box<dyn Future<Item=Response<Body>, Error=Error> + Send>
{
    let resp = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    Box::new(future::ok(resp))
}


fn main() {
    let files = Path::new("./files");
    fs::create_dir(files).ok();
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(move || {
        service_fn(move |req| microservice_handler(req, &files))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
