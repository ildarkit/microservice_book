use std::thread;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender};
use std::collections::HashMap;
use anyhow::Error;
use lettre::error::Error as LettreError;
use lettre::address::{Address, Envelope, AddressError};
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
#[macro_use]
extern crate nickel;
use nickel::{Nickel, HttpRouter, FormBody, Request, Response,
    MiddlewareResult};
use nickel::status::StatusCode;
use nickel::template_cache::{ReloadPolicy, TemplateCache};
use log::error;

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

    let from: Mailbox = "<admin@example.com>".parse()?;
    let to = to.parse::<Address>()?;
    let envelope = Envelope::new(None, vec![to])?;
    let email = Message::builder()
        .envelope(envelope)
        .subject("Confirm email".to_string())
        .from(from)
        .body(body)?;

    let sender = data.sender.lock().unwrap().clone();
    sender.send(email)
        .map_err(|_| Error::msg("Can't send message to worker"))?;
    Ok(())
}

fn spawn_sender() -> Sender<Message> {  
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mailer = SmtpTransport::builder_dangerous("localhost:2525")
            .credentials(
                Credentials::new("admin@example.com".into(), "password".into())
            )
            .build();   
        for email in rx.iter() {
            let result = mailer.send(&email);
            if let Err(err) = result {
                error!("Can't send mail: {}", err);
            }
        }
    });
    tx
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let tx = spawn_sender();

    let data = Data {
        sender: Mutex::new(tx),
        cache: TemplateCache::with_policy(ReloadPolicy::Always),
    };
    let mut server = Nickel::with_data(data);
    server.get("/", middleware!("Mailer microservice"));
    server.post("/send", send);
    server.listen("127.0.0.1:8002").unwrap();
}
