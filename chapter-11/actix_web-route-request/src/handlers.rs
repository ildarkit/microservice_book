use serde_derive::{Deserialize, Serialize};
use actix_web::{Error, HttpRequest, HttpResponse, web::Form};
use actix_web::http::{header, StatusCode};
use std::future::Future;
use failure::format_err;
use anyhow::{Result, Context};

use crate::client;
use crate::middleware::RequestCount;

#[derive(Deserialize, Serialize)]
pub struct UserForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct UserId {
    id: String,
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

pub async fn signup(params: Form<UserForm>) -> Result<HttpResponse> {
    let res = client::post_request::<UserForm, ()>("http://127.0.0.1:8080/signup", 
                           params.into_inner()) 
        .await
        .map(|res| {
            match res {
                Ok(_) => {
                    HttpResponse::Found()
                        .insert_header((header::LOCATION, "/login.html"))
                        .finish()
                },
                Err(err) => {
                    
                },
            }
        });
    res
}

pub async fn signin(req: HttpRequest, params: Form<UserForm>)
    -> Result<HttpResponse, Error>
{
    let res = client::post_request("http://127.0.0.1:8080/signin",
                           params.into_inner())
        .map(move |id: UserId| {
            req.remember(id.id);
            HttpResponse::build_from(&req)
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/comments.html")
                .finish()
        });
    Ok(res)
}

pub async fn new_comment(req: HttpRequest, params: Form<AddComment>)
    -> Result<HttpResponse, Error>
{
    let res = req.identity()
        .ok_or(format_err!("not authorized").into())
        .into_future()
        .and_then(move |uid| {
            let params = NewComment {
                uid,
                text: params.into_inner().text,
            };
            client::post_request::<_, ()>("http:/127.0.0.1:8080/new_comment",
                                  params)
        })
    .then(move |_| {
        HttpResponse::build_from(&req)
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/comments.html")
            .finish()
    });
    Ok(res)
}

pub async fn comments(_req: HttpRequest) -> Result<HttpResponse, Error> {
    let res = client::get_request("http://127.0.0.1:8080/comments")
        .map(|data| {
            HttpResponse::Ok().body(data)
        });
    Ok(res)
}

pub fn counter(req: HttpRequest, req_count: RequestCount) -> String {
    format!("{}", req_count.0.borrow())
}
