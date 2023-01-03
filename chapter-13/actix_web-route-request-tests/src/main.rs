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

#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::Mutex;
    use std::time::Duration;
    use lazy_static::lazy_static;
    use mockito::{mock, Mock};
    use request::Client;
    use serde::{Deserialize, Serialize};
    use super::*;

    #[derive(Deserialize, Serialize)]
    struct Comment {
        id: Option<i32>,
        uid: String,
        text: String,
    } 

    fn add_mock<T>(method: &str, path: &str, result: T) -> Mock
        where 
            T: Serialize,
    {
        mock(method, path)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(&result).unwrap())
            .create()
    }

    lazy_static! {
        static ref STARTED: Mutex<bool> = Mutex::new(false);
    }

    fn setup() {
        let mut started = STARTED.lock().unwrap();
        if !*started {
            thread::spawn(|| {
                let url = mockito::server_url();
                let _signup = add_mock("POST", "/signup", ());
                let _signin = add_mock(
                    "POST", "/signin", UserId { id: "user-id".into() });
                let _new_comment = add_mock("POST", "/new_comment", ());
                let comment = Comment {
                    id: None,
                    text: "comment".into(),
                    uid: "user-id".into(),
                };
                let _comment = add_mock("GET", "/comments", vec![comment]);
                let links = LinksMap {
                    signup: mock_url(&url, "/signup"),
                    signin: mock_url(&url, "/signin"),
                    new_comment: mock_url(&url, "/new_comment"),
                    comments: mock_url(&url, "/comments"),
                };
                start(links);
            });
            thread::sleep(Duration::from_secs(5));
            *started = true;
        }
    }

    fn mock_url(base: &str, path: &str) -> String {
        format!("{}{}", base, path)
    }

    fn start(links: LinksMap) {
        let sys = actix::System::new("router");
        let state = State {
            counter: Refcell::default(),
            links,
        };
        sys.block_on(
            HttpServer::new(move || {
                let data = web::Data::new(state);
                App::new()
                    .wrap(IdentityMiddleware::default())
                    .wrap(
                        SessionMiddleware::builder(
                            CookieSessionStore::default(),
                            secret.clone()
                        )
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
            .bind("127.0.0.1:8080").unwrap().run()
        ).unwrap();
        sys.run().unwrap();
    }

    fn test_get<T>(path: &str) -> T
    where
        T: for <'de> Deserialize <'de>,
    {
        let client = Client::new();
        let data = client.get(&test_url(path))
            .send()
            .unwrap()
            .text()
            .unwrap();
        serde_json::from_str(&data).unwrap()
    }

    fn test_url(path: &str) -> String {
        format!("http://127.0.0.1:8080/api{}", path)
    }

    fn test_post<T>(path: &str, data: &T)
        where
            T: Serialize,
    {
        setup();
        let client = Client::new();
        let resp = client.post(&test_url(path))
            .form(data)
            .send()
            .unwrap();
        let status = resp.status();
        assert!(status.is_success());
    }

    #[test]
    fn test_signup_with_client() {
        let user = UserForm {
            email: "abc@example.com".into(),
            password: "abc".into(),
        };
        test_post("/signup", &user);
    }

    #[test]
    fn test_signin_with_client() {
        let user = UserForm {
            email: "abc@example.com".into(),
            password: "abc".into(),
        };
        test_post("/signin", &user);
    }

    #[test]
    fn test_list_with_client() {
        let _ = Vec<Comment> = test_get("/comments");
    }
}
