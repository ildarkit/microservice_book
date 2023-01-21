use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| App::new().service(greet).wrap(Logger::default()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
