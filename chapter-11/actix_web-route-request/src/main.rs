use actix_files as fs;
use actix::prelude::SyncArbiter;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};

use request_count::counter::CountState;
use request_count::middleware::Counter;
use request_count::handlers;
use request_count::cache::{CacheLink, CacheActor};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let secret = Key::generate();

    let addr = SyncArbiter::start(3, || {
        CacheActor::new("redis://127.0.0.1:6379", 10)
    });
    let cache = CacheLink::new(addr); 

    HttpServer::new(move || {
        let state = CountState::new(cache.clone());
        let data = web::Data::new(state);
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
            .app_data(web::Data::clone(&data))
            .service(
                web::scope("/api")
                    .route("/singup", web::post().to(handlers::signup))
                    .route("/signin", web::post().to(handlers::signin))
                    .route("/new_comment", web::post().to(handlers::new_comment))
                    .route("/comments", web::get().to(handlers::comments))
            )
            .route("stats/counter", web::get().to(handlers::counter))
            .service(
                fs::Files::new("/", "./static").index_file("index.html")
            )
    })
    .workers(1)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
