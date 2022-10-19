#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
//extern crate base64;
#[macro_use]
extern crate base64_serde;

mod color;

use hyper::{Body, Response, Server, Error, Method, Request, StatusCode};
use hyper::service::service_fn;
use futures::{Stream, Future};
use rand::Rng;
use rand::distributions::{Uniform, Normal, Bernoulli};
use core::ops::Range;
use std::cmp::{min, max};
use base64::STANDARD;
use color::Color;


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
    -> impl Future<Item=Response<Body>, Error=Error>
{
    let method = req.method();
    let path = req.uri().path();
    match (method, path) {
        (&Method::POST, "/random") => {
            let body = req.into_body().concat2()
                .map(|chunks| {
                    let res = serde_json::from_slice::<RngRequest>(chunks.as_ref())
                        .map(handle_request)
                        .and_then(|resp| serde_json::to_string(&resp));
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
        _ => {
            todo!()
        }
    }
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
