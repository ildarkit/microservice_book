use std::error::Error;
use thiserror;
use anyhow;
use serde::Serialize;
use awc::error::SendRequestError;
use actix_web::{self, error::ResponseError, http::StatusCode, HttpResponse};

use crate::client::ClientHttpError;

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
    cause: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("{1}")]
    ClientError(#[source] ClientHttpError, String),
    #[error(transparent)]
    UserIdentityError(#[from] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] actix_web::Error),
}

impl ApiError {
    fn new(e: ClientHttpError, ctx: &str) -> Self {
        ApiError::ClientError(
            e,
            ctx.into()
        )
    }

    fn get_cause(&self) -> String {
        match self.source() {
            Some(s) => s.to_string(),
            None => "Unknown".into()
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ClientError(
                ClientHttpError::SendError(SendRequestError::Url(_)),
                _
            ) => {
                StatusCode::BAD_REQUEST
            },
            ApiError::UnexpectedError(err) => {
                let err = err.as_response_error();
                err.status_code()
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(ApiErrorResponse {
                error: self.to_string(),
                cause: self.get_cause(),
            })
    } 
}

pub fn context_err(e: ClientHttpError, ctx: &str) -> ApiError {
    ApiError::new(e, ctx)
}
