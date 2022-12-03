use actix_files as fs;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};

use request_count::middleware::Counter;
use request_count::handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let secret = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_name("auth-example".to_owned())
                    .cookie_secure(false)
                    .build()
            )
            .wrap(Logger::default())
            .wrap(Counter)
            .service(
                web::scope("/api")
                    .route("/singup", web::post().to(handlers::signup))
                    .route("/signin", web::post().to(handlers::signin))
                    .route("/new_comment", web::post().to(handlers::new_comment))
                    .route("/comments", web::get().to(handlers::comments))
            )
            .route("stats/counter", web::get().to(handlers::counter))
            .service(
                fs::Files::new("/static/", ".").index_file("index.html")
            )
    })
    .workers(1)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
