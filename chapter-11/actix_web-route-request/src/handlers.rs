use serde_derive::{Deserialize, Serialize};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web, Responder};
use actix_identity::Identity;
use actix_web::http::header;

use crate::client;
use crate::counter::CountState;
use crate::error::{ApiError, context_err};

#[derive(Deserialize, Serialize)]
pub struct UserForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct UserId {
    id: Option<String>,
}

#[derive(Deserialize)]
pub struct AddComment {
    pub text: String,
}

#[derive(Serialize)]
pub struct NewComment {
    pub uid: String,
    pub text: String,
}

fn redirect(url: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, url))
        .finish()
}

pub async fn signup(params: web::Form<UserForm>) -> Result<impl Responder, ApiError> {
    client::post_request::<UserForm, _>(
        "http://127.0.0.1:8001/signup",
        params.into_inner()
    )
    .await
    .map_err(|e| context_err(e, "Failed to signup"))?;
    
    Ok(redirect("/login.html"))
}

pub async fn signin(req: HttpRequest, params: web::Form<UserForm>)
    -> Result<impl Responder, ApiError>
{
    let user = client::post_request::<UserForm, UserId>(
        "http://127.0.0.1:8001/signin",
        params.into_inner()
    )
    .await
    .map_err(|e| context_err(e, "Failed to signin"))?;

    let mut url = "/comments.html";
    match user.id {
        Some(user_id) => { 
            Identity::login(&req.extensions(), user_id)?;
        },
        None => {
            url = "/login.html";
        },
    }
    
    Ok(redirect(url))
}

pub async fn new_comment(
    params: web::Form<AddComment>,
    user: Option<Identity>
)
    -> Result<impl Responder, ApiError>
{
    let mut url = "/comments.html";

    if let Some(user) = user {
        let params = NewComment {
            uid: user.id()?,
            text: params.into_inner().text,
        };
        client::post_request::<_, ()>(
            "http:/127.0.0.1:8003/new_comment",
            params
        )
        .await
        .map_err(|e| context_err(e, "Failed to add comment"))?;
    } else {
        url = "/login.html";
    }

    Ok(redirect(url))
}

pub async fn comments(_req: HttpRequest, count_state: web::Data<CountState>)
    -> Result<impl Responder, ApiError>
{
    let fut = client::get_request("http://127.0.0.1:8003/comments");
    let data = count_state.cache("/list", fut)
        .await
        .map_err(|e| context_err(e, "Failed to get comments"))?;

    Ok(HttpResponse::Ok().body(data))
}

pub async fn counter(count_state: web::Data<CountState>) -> impl Responder {
    format!("{}", count_state.get_ref())
}
