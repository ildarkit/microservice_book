mod settings;

use std::thread;
use std::path::Path;
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
    from: String,
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

fn check_file(path: &str) -> Result<&Path> {
    let path = Path::new(path);
    match path.exists() && path.is_file() {
        true => Ok(path),
        false => {
            let path = path.to_str().unwrap();
            Err(Error::msg(format!("No such path to template file: {path}")))
        },  
    } 
}

fn build_smtp_transport(
    address: String,
    login: Option<String>,
    password: Option<String>
) -> Result<SmtpTransport> 
{
    let smtp_address: Vec<_> = address
        .to_socket_addrs()
        .map_err(|_| Error::msg("Unable to parse address: {address}"))?
        .collect();
    debug!("Smtp relay address: {}:{}", smtp_address[0].ip(), smtp_address[0].port());

    let smtp = SmtpTransport::builder_dangerous(
                format!("{}", smtp_address[0].ip())
            )
            .port(smtp_address[0].port());

    let smtp = match (login, password) {
        (Some(login), Some(password)) => {
            smtp.credentials(
                Credentials::new(
                    login,
                    password,
                )
            )
            .authentication(vec![Mechanism::Plain]) 
            .build()
        },
        _ => {
            smtp.build()
        },
    };
    Ok(smtp)
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

    let path = check_file("./templates/confirm.tpl")?;
    let data =req.server_data();
    data.cache.render(
        path,
        &mut body,
        &params
    )?;
    debug!("Body: {:?}", body);

    let email = Message::builder()
        .subject("Confirm email".to_string())
        .from(data.from.parse()?)
        .to(to.parse()?)
        .body(body)?;
    debug!("Mail: {}", std::str::from_utf8(&email.formatted()).unwrap());

    let sender = data.sender.lock().unwrap().clone();
    sender.send(email)
        .map_err(|_| Error::msg("Can't send message to worker"))?;
    Ok(())
}

fn spawn_sender(mailer: SmtpTransport)
    -> Result<Sender<Message>>
{  
    let (tx, rx) = channel::<Message>();

    thread::spawn(move || { 
        for email in rx.iter() {
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
    let mailer = build_smtp_transport(
        conf.smtp_address,
        conf.smtp_login,
        conf.smtp_password
    )?;

    let tx = spawn_sender(mailer)?;

    let data = Data {
        sender: Mutex::new(tx),
        cache: TemplateCache::with_policy(ReloadPolicy::Always),
        from: conf.from_address,
    };
    let mut server = Nickel::with_data(data);
    server.get("/", middleware!("Mailer microservice"));
    server.post("/send", send);
    server.listen(conf.address).unwrap();
    Ok(())
}
