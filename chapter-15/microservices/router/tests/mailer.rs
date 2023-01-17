mod utils;

use self::utils::*;

#[test]
fn mails_healthcheck() {
    let mut api = WebApi::mailer();
    api.healthcheck("/", "Mailer microservice");
}

#[test]
fn send_mail() {
    let mut api = WebApi::mailer();
    let mut email = rand_str() + "@example.com";
    email = email.to_lowercase();
    let code = rand_str();
    let params = vec![
        ("to", email.as_ref()),
        ("code", code.as_ref()),
    ];
    let sent: bool = api.request(Method::POST, "/send", params);
    assert!(sent);
}
