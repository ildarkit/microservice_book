mod cache;
mod client;
mod handlers;
mod error;
mod cache_actor;
mod settings;

use anyhow::{Result, Error};
use actix_files as fs;
use actix::prelude::{System, SyncArbiter};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};

use cache_actor::{CacheLink, CacheActor};
use settings::Settings;

fn start(links: Settings) -> Result<()> {
    let sys = System::new();

    sys.block_on(
        async move {
            let secret = Key::generate();
            
            let redis = links.redis.clone();
            let addr = SyncArbiter::start(3, move || { 
                CacheActor::new(&redis, 10)
            });
            let cache = CacheLink::new(addr);
            let bind_address = links.address.clone();

            HttpServer::new(move || {
                let data = web::Data::new(links.clone());
                let cache = web::Data::new(cache.clone());
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
                    .app_data(data)
                    .app_data(cache)
                    .service(
                        web::scope("/api")
                            .route("/signup", web::post().to(handlers::signup))
                            .route("/signin", web::post().to(handlers::signin))
                            .route("/new_comment", web::post().to(handlers::new_comment))
                            .route("/comments", web::get().to(handlers::comments))
                    )
                    .service(
                        fs::Files::new("/", "./static").index_file("index.html")
                    )
            })
            .workers(1)
            .bind(&bind_address)?
            .run()
            .await
        }
    )?;
    sys.run().map_err(|_| Error::msg("Can't run actors system"))
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let links = Settings::new()?;
    start(links)
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::{Mutex, RwLock};
    use std::time::Duration;
    use std::borrow::BorrowMut;
    use lazy_static::lazy_static;
    use mockito::{mock, Mock};
    use reqwest::blocking::Client;
    use serde::{Deserialize, Serialize};
    use log::debug;
    use super::*;
    use super::handlers::*;

    static mut LINKS: Option<RwLock<Settings>> = None;

    #[derive(Deserialize, Serialize)]
    struct Comment {
        id: Option<i32>,
        uid: String,
        text: String,
    } 
    
    fn get_links() -> Settings {
        unsafe { LINKS.as_ref().unwrap().read().unwrap().clone() }
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
                unsafe {
                    *LINKS.borrow_mut() = 
                        Some(RwLock::new(
                            Settings {
                                signup: mock_url(url, "/signup"),
                                signin: mock_url(url, "/signin"),
                                new_comment: mock_url(url, "/new_comment"),
                                comments: mock_url(url, "/comments"),
                                address: "127.0.0.1:8080".into(),
                                redis: "redis://127.0.0.1:6379".into(),
                            })
                        );
                }
                let links = get_links();
                debug!("Mock links: {:#?}", &links);
                start(links).unwrap();
            });
            thread::sleep(Duration::from_secs(1));
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
        setup();
        let path = &test_api_url(path);
        debug!("GET request to: {path}");
        client_get(path)
    }

    fn client_get<T>(path: &str) -> T
    where
        T: for <'de> Deserialize <'de>,
    {
        let client = Client::new();
        let data = client.get(path)
            .send()
            .unwrap()
            .text()
            .unwrap();
        serde_json::from_str(&data).unwrap()
    }

    fn test_api_url(path: &str) -> String {
        let links = get_links(); 
        format!("http://{}/api{path}", links.address)
    }

    fn test_api_post<T>(path: &str, data: &T)
        where
            T: Serialize,
    {
        setup();
        let path = &test_api_url(path);
        debug!("POST request to: {path}");
        client_post(path, data);
    }

    fn client_post<T>(path: &str, data: &T)
        where
            T: Serialize,
    {
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
}
