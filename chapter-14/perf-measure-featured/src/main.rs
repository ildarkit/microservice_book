use std::thread;
use std::time::Duration;
use std::sync::{RwLock, Mutex, Arc};
use chrono::Utc;
use askama::Template;
use actix_web::{middleware::Logger, web, App, HttpServer, HttpResponse};

fn now() -> String {
    Utc::now().to_string()
}

#[cfg(not(feature = "borrow"))]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    time: String,
}

#[cfg(feature = "borrow")]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    time: &'a str,
}

#[derive(Clone)]
struct State {
    #[cfg(not(feature = "rwlock"))]
    last_minute: Arc<Mutex<String>>,
    #[cfg(feature = "rwlock")]
    last_minute: Arc<RwLock<String>>,
    cached: Arc<RwLock<Option<String>>>,
}

async fn index(state: web::Data<State>) -> HttpResponse {
    if cfg!(feature = "cached") {
        let cached = state.cached.read().unwrap();
        if let Some(ref body) = *cached {
            return HttpResponse::Ok().body(body.to_owned());
        }
    }

    #[cfg(not(feature = "rwlock"))]
    let last_minute = state.last_minute.lock().unwrap();
    #[cfg(feature = "rwlock")]
    let last_minute = state.last_minute.read().unwrap();

    #[cfg(not(feature = "borrow"))]
    let template = IndexTemplate { time: last_minute.to_owned() };
    #[cfg(feature = "borrow")]
    let template = IndexTemplate { time: &last_minute };
    let body = template.render().unwrap();

    if cfg!(feature = "cached") {
        let mut cached = state.cached.write().unwrap();
        *cached = Some(body.clone());
    }

    HttpResponse::Ok().body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let value = now();
    #[cfg(not(feature = "rwlock"))]
    let last_minute = Arc::new(Mutex::new(value));
    #[cfg(feature = "rwlock")]
    let last_minute = Arc::new(RwLock::new(value));

    let last_minute_ref = last_minute.clone();
    thread::spawn(move || {
        loop {
            {
                #[cfg(not(feature = "rwlock"))]
                let mut last_minute = last_minute_ref.lock().unwrap();
                #[cfg(feature = "rwlock")]
                let mut last_minute = last_minute_ref.write().unwrap();
                *last_minute = now();
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    let cached = Arc::new(RwLock::new(None));
    let state = State {
        last_minute,
        cached,
    };

    HttpServer::new(move || {
        let data = web::Data::new(state.clone());
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
    
    Ok(())
}
