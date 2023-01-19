use log::debug;
use anyhow::Error;
use serde_derive::Serialize;
use rouille::{Request, Response, router, post_input};
use diesel::{select, dsl::exists};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use crypto::pbkdf2::{pbkdf2_simple, pbkdf2_check};
use user_models::{models, schema};

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[derive(Serialize)]
struct UserId {
    id: Option<String>,
}

type StatusCode = u16;

enum ResponseStatus<T: serde::Serialize> {
    Text((String, StatusCode)),
    Json((T, StatusCode)),
    Empty404,
    Created201,
}

pub fn handler(request: &Request, pool: &Pool) -> Result<Response, Error> {
    debug!("{:?}", request);
    let response = router!(request, 
        (GET) (/) => {
            ResponseStatus::Text(("Users Microservice".to_string(), 200))
        },
        (POST) (/signup) => {
            let data = post_input!(request, {
                email: String,
                password: String,
            })?;
            let user_email = data.email.trim().to_lowercase();
            let user_pass = pbkdf2_simple(&data.password, 12345)?;
            {
                use self::schema::users::dsl::*;

                let mut conn = pool.get()?;
                let user_exists: bool = select(
                    exists(users.filter(email.eq(user_email.clone())))
                )
                    .get_result(&mut conn)?;
                if !user_exists {
                    let uuid = format!("{}", uuid::Uuid::new_v4());
                    let new_user = models::NewUser {
                        id: &uuid,
                        email: &user_email,
                        password: &user_pass,
                    };
                    diesel::insert_into(schema::users::table)
                        .values(&new_user)
                        .execute(&mut conn)?;
                    ResponseStatus::Created201
                } else {
                    ResponseStatus::Text(
                        (format!("user {} exists", data.email), 400))
                }
            }
        },
        (POST) (/signin) => {
            let data = post_input!(request, {
                email: String,
                password: String,
            })?;
            let user_email = data.email.trim().to_lowercase();
            let user_pass = data.password;
            debug!("Signin user: {user_email} {user_pass}");
            {
                use self::schema::users::dsl::*;
 
                let mut resp_code = 403;
                let mut user_id = UserId { id: None };
                let mut conn = pool.get()?;
                let user = users
                    .filter(email.eq(user_email))
                    .first::<models::User>(&mut conn)
                    .ok();
                if let Some(user) = user {
                    debug!("Fetched database user: {:?}", user);
                    let valid = pbkdf2_check(&user_pass, &user.password)
                        .map_err(|err| Error::msg(format!("pass check error {err}")))?;
                    if valid {
                        debug!("User is valid");
                        resp_code = 200;
                        user_id = UserId { id: Some(user.id) };
                    };
                };
                ResponseStatus::Json((user_id, resp_code))
            }
        },
        _ => {
            ResponseStatus::Empty404
        }
    ); 
    let response = match response {
        ResponseStatus::Text((text, code)) => {
            Response::text(text)
                .with_status_code(code)
        },
        ResponseStatus::Json((resp, code)) => {
            Response::json(&resp)
                .with_status_code(code)
        },
        ResponseStatus::Empty404 => {
            Response::empty_404()
        },
        ResponseStatus::Created201 => {
            Response::json(&())
                .with_status_code(201)
        },
    };
    debug!("{:?}", response);
    Ok(response)
}
