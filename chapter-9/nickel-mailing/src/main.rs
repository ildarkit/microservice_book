mod settings;

use std::thread;
use std::sync::Mutex;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{channel, Sender};
use std::collections::HashMap;
use anyhow::{Result, Error};
use lettre::error::Error as LettreError;
use lettre::address::AddressError;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::{
    authentication::{Credentials, Mechanism}};
#[macro_use]
extern crate nickel;
use nickel::{Nickel, HttpRouter, FormBody, Request, Response,
    MiddlewareResult};
use nickel::status::StatusCode;
use nickel::template_cache::{ReloadPolicy, TemplateCache};
use log::{debug, error};
use settings::Settings;

struct Data {
    sender: Mutex<Sender<Message>>,
    cache: TemplateCache,
}

#[derive(thiserror::Error, Debug)]
enum MailError {
    #[error("{0}")]
    FormError(String),
    #[error(transparent)]
    TemplateError(#[from] mustache::Error),
    #[error(transparent)]
    MessageError(#[from] LettreError),
    #[error(transparent)]
    AddressError(#[from] AddressError),
    #[error(transparent)]
    OtherError(#[from] Error),
}

fn send<'mw>(req: &mut Request<Data>, res: Response<'mw, Data>) 
    -> MiddlewareResult<'mw, Data>
{
    try_with!(res, send_impl(req).map_err(|e| {
        error!("Failed to send email:\n\tCause: {e}");
        match e {
            MailError::FormError(_) => StatusCode::BadRequest,
            _ => StatusCode::InternalServerError,
        }
    }));
    res.send("true")
}

fn send_impl(req: &mut Request<Data>) -> Result<(), MailError> {
    let (to, code) = {
        let params = req.form_body()
            .map_err(|_| MailError::FormError("Can't get form body".into()))?;
        let to = params.get("to")
            .ok_or(MailError::FormError("<TO> field not set".into()))?
            .to_owned();
        let code = params.get("code")
            .ok_or(MailError::FormError("<CODE> field not set".into()))?
            .to_owned();
        (to, code)
    };
     
    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("code", &code);
    let mut body: Vec<u8> = Vec::new();

    let data =req.server_data();
    data.cache.render(
        "templates/confirm.tpl",
        &mut body,
        &params
    )?;

    let email = Message::builder()
        .subject("Confirm email".to_string())
        .from("<admin@example.com>".parse()?)
        .to(to.parse()?)
        .body(body)?;

    let sender = data.sender.lock().unwrap().clone();
    sender.send(email)
        .map_err(|_| Error::msg("Can't send message to worker"))?;
    Ok(())
}

fn spawn_sender(
    address: String,
    login: Option<String>,
    password: Option<String>)
    -> Result<Sender<Message>>
{  
    let (tx, rx) = channel::<Message>();

    let smtp_address: Vec<_> = address
        .to_socket_addrs()
        .map_err(|_| Error::msg("Unable to parse address: {address}"))?
        .collect();

    thread::spawn(move || {
        let mailer = SmtpTransport::builder_dangerous(
                format!("{}", smtp_address[0].ip())
            )
            .port(smtp_address[0].port());
        let mailer = if login.is_none() {
            mailer.build()
        } else {
            mailer.credentials(Credentials::new(
                login.unwrap(),
                password.unwrap(),
            ))
            .authentication(vec![Mechanism::Plain]) 
            .build()
        };

        for email in rx.iter() {
            debug!("{}", std::str::from_utf8(&email.formatted()).unwrap());
            let result = mailer.send(&email);
            if let Err(err) = result {
                error!("Can't send mail: {}", err);
            }
        }
    });
    Ok(tx)
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let conf = Settings::new()?;
    let tx = spawn_sender(
        conf.smtp_address,
        conf.smtp_login,
        conf.smtp_password
    )?;

    let data = Data {
        sender: Mutex::new(tx),
        cache: TemplateCache::with_policy(ReloadPolicy::Always),
    };
    let mut server = Nickel::with_data(data);
    server.get("/", middleware!("Mailer microservice"));
    server.post("/send", send);
    server.listen(conf.address).unwrap();
    Ok(())
}
