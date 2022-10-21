#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate base64;
#[macro_use]
extern crate base64_serde;
extern crate queryst;
extern crate serde_cbor;

mod color;

use hyper::{Body, Response, Server, Method, Request, StatusCode};
use hyper::service::service_fn;
use futures::{future, Stream, Future};
use rand::Rng;
use rand::distributions::{Uniform, Normal, Bernoulli};
use core::ops::Range;
use std::cmp::{min, max};
use base64::STANDARD;
use color::Color;
use failure::Error;
use serde_json::Value;


#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum RngResponse {
    Value(f64),
    #[serde(with = "Base64Standard")]
    Bytes(Vec<u8>),
    Color(Color),
}


#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
    Shuffle {
        #[serde(with = "Base64Standard")]
        data: Vec<u8>,
    },
    Color {
        from: Color,
        to: Color,
    },
}


base64_serde_type!(Base64Standard, STANDARD);


fn handler(req: Request<Body>)
    -> Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    let method = req.method();
    let path = req.uri().path();
    match (method, path) {
        (&Method::POST, "/random") => {
            let format = {
                let uri = req.uri().query().unwrap_or("");
                let query = queryst::parse(uri).unwrap_or(Value::Null);
                query["format"].as_str().unwrap_or("json").to_string()
            };
            let body = req.into_body().concat2()
                .map(move |chunks| {
                    let res = serde_json::from_slice::<RngRequest>(chunks.as_ref())
                        .map(handle_request)
                        .map_err(Error::from)
                        .and_then(move |resp| serialize(&format, &resp));
                    match res {
                        Ok(body) => {
                            Response::new(body.into())
                        },
                        Err(err) => {
                            Response::builder()
                                .status(StatusCode::UNPROCESSABLE_ENTITY)
                                .body(err.to_string().into())
                                .unwrap()
                        },
                    }
                });
            Box::new(body)
        },
        (&Method::POST, _) => {
            response_with_code(StatusCode::NOT_FOUND)
        },
        (&Method::GET, "/") | (&Method::GET, "/random") => {
            Box::new(future::ok(Response::new("Random microservice".into())))
        },
        _ => {
            response_with_code(StatusCode::METHOD_NOT_ALLOWED)
        },
    }
}


fn response_with_code(status_code: StatusCode)
    -> Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    let body = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    Box::new(future::ok(body))
}


fn handle_request(request: RngRequest) -> RngResponse {
    let mut rng = rand::thread_rng();
    match request {
        RngRequest::Uniform { range } => {
            let value = rng.sample(Uniform::from(range)) as f64;
            RngResponse::Value(value)
        },
        RngRequest::Normal { mean, std_dev } => {
            let value = rng.sample(Normal::new(mean, std_dev)) as f64;
            RngResponse::Value(value)
        },
        RngRequest::Bernoulli { p } => {
            let value = rng.sample(Bernoulli::new(p)) as i8 as f64;
            RngResponse::Value(value)
        },
        RngRequest::Shuffle { mut data } => {
            rng.shuffle(&mut data);
            RngResponse::Bytes(data)
        },
        RngRequest::Color { from, to } => {
            let red = rng.sample(color_range(from.red, to.red));
            let green = rng.sample(color_range(from.green, to.green));
            let blue = rng.sample(color_range(from.blue, to.blue));
            RngResponse::Color(Color { red, green, blue })
        },
    }
}


fn color_range(from: u8, to: u8) -> Uniform<u8> {
    let (from, to) = (min(from, to), max(from, to));
    Uniform::new_inclusive(from, to)
}


fn serialize(format: &str, resp: &RngResponse) -> Result<Vec<u8>, Error> {
    match format {
        "json" => {
            Ok(serde_json::to_vec(resp)?)
        },
        "cbor" => {
            Ok(serde_cbor::to_vec(resp)?)
        },
        _ => {
            Err(format_err!("unsupported format {}", format))
        },
    }
}


fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn(|req| {
            handler(req)
        })
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
