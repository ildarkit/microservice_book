mod utils;

#[cfg(test)]
mod mailer_test {
    use super::utils::{Method, WebApi};

    #[test]
    fn mails_healthcheck() {
        let mut api = WebApi::mailer();
        api.healthcheck("/", "Mailer Microservice");
    }

    #[test]
    fn send_mail() {
        let mut api = WebApi::mailer();
        let email = utils::rand_str() + "@example.com";
        let code = utils::rand_str();
        let params = vec![
            ("to", emails.as_ref()),
            ("code", code.as_ref()),
        ];
        let sent: bool = api.request(Method::POST, "/send", params);
        assert!(sent);
    }
}
