use actix_files as fs;
use actix::prelude::{System, SyncArbiter};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};

use request_count::LinksMap;
use request_count::counter::CountState;
use request_count::middleware::Counter;
use request_count::handlers;
use request_count::cache::{CacheLink, CacheActor};

fn read_config() -> std::io::Result<LinksMap> {
    let links = std::fs::read_to_string("links.toml")?;
    Ok(toml::from_str(&links)?)
}

fn start(links: &LinksMap) -> std::io::Result<()> {
    let sys = System::new();

    sys.block_on(
        async move {
            let secret = Key::generate();
            let addr = SyncArbiter::start(3, || {
                CacheActor::new("redis://127.0.0.1:6379", 10)
            });
            let cache = CacheLink::new(addr);
            let links = links.clone();

            HttpServer::new(move || {
                let state = CountState::new(cache.clone(), links.clone());
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
                            .route("/signup", web::post().to(handlers::signup))
                            .route("/signin", web::post().to(handlers::signin))
                            .route("/new_comment", web::post().to(handlers::new_comment))
                            .route("/comments", web::get().to(handlers::comments))
                    )
                    .route("/stats/counter", web::get().to(handlers::counter))
                    .service(
                        fs::Files::new("/", "./static").index_file("index.html")
                    )
            })
            .workers(1)
            .bind("127.0.0.1:8080")?
            .run()
            .await
        }
    )?;
    sys.run()
}

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let links = &read_config()?;
    start(links)
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::Mutex;
    use lazy_static::lazy_static;
    use mockito::{mock, Mock};
    use reqwest::blocking::Client;
    use serde::{Deserialize, Serialize};
    use log::debug;
    use super::*;
    use super::handlers::*;

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
                env_logger::init();
                let url = &mockito::server_url();
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
                let _count = add_mock("GET", "/stats/counter", 1);
                let links = &LinksMap {
                    signup: mock_url(url, "/signup"),
                    signin: mock_url(url, "/signin"),
                    new_comment: mock_url(url, "/new_comment"),
                    comments: mock_url(url, "/comments"),
                };
                debug!("Mock links: {:#?}", links);
                start(links).unwrap();
            });
            *started = true;
        }
    }

    fn mock_url(base: &str, path: &str) -> String {
        format!("{}{}", base, path)
    }

    fn test_api_get<T>(path: &str) -> T
    where
        T: for <'de> Deserialize <'de>,
    { 
        let path = &test_api_url(path);
        debug!("GET request to: {path}");
        client_get(path)
    }

    fn test_get<T>(path: &str) -> T
    where
        T: for <'de> Deserialize <'de>,
    { 
        debug!("GET request to: {path}");
        let path = &test_url(path);
        client_get(path)
    }

    fn client_get<T>(path: &str) -> T
    where
        T: for <'de> Deserialize <'de>,
    {
        setup();
        let client = Client::new();
        let data = client.get(path)
            .send()
            .unwrap()
            .text()
            .unwrap();
        serde_json::from_str(&data).unwrap()
    }

    fn test_api_url(path: &str) -> String {
        format!("http://127.0.0.1:8080/api{}", path)
    }

    fn test_url(path: &str) -> String {
        format!("http://127.0.0.1:8080{}", path)
    }

    fn test_api_post<T>(path: &str, data: &T)
        where
            T: Serialize,
    {
        let path = &test_api_url(path);
        debug!("POST request to: {path}");
        client_post(path, data);
    }

    fn client_post<T>(path: &str, data: &T)
        where
            T: Serialize,
    {
        setup();
        let client = Client::new();
        let resp = client.post(path)
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
        test_api_post("/signup", &user);
    }

    #[test]
    fn test_signin_with_client() {
        let user = UserForm {
            email: "abc@example.com".into(),
            password: "abc".into(),
        };
        test_api_post("/signin", &user);
    }

    #[test]
    fn test_list_with_client() {
        let _: Vec<Comment> = test_api_get("/comments");
    }

    #[test]
    fn test_new_comment() {
        let comment = NewComment {
            uid: "user-id".into(),
            text: "new comment".into(),
        };
        test_api_post("/new_comment", &comment);
    }

    #[test]
    fn test_stats_counter() {
        let _:i64 = test_get("/stats/counter");
    }
}
