extern crate futures;
extern crate hyper;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use hyper::{Body, Response, Server, Error, Method, Request, StatusCode};
use hyper::service::service_fn;
use futures::{Stream, Future};
use rand::Rng;
use rand::distributions::{Uniform, Normal, Bernoulli};
use core::ops::Range;


#[derive(Serialize)]
struct RngResponse {
    value: f64,
}

#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
}


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
    let value = {
        match request {
            RngRequest::Uniform { range } => {
                rng.sample(Uniform::from(range)) as f64
            },
            RngRequest::Normal { mean, std_dev } => {
                rng.sample(Normal::new(mean, std_dev)) as f64
            },
            RngRequest::Bernoulli { p } => {
                rng.sample(Bernoulli::new(p)) as i8 as f64
            },
        }
    };
    RngResponse { value }
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
