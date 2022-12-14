use std::error::Error;
use thiserror;
use anyhow;
use serde::Serialize;
use awc::error::SendRequestError;
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};

use crate::client::ClientHttpError;

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
    cause: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("{1}")]
    HttpError(#[source] ClientHttpError, String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ApiError {
    fn new(e: ClientHttpError, ctx: &str) -> Self {
        ApiError::HttpError(
            e,
            ctx.into()
        )
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::HttpError(
                ClientHttpError::SendError(SendRequestError::Url(_)),
                _
            ) => {
                StatusCode::BAD_REQUEST
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(ApiErrorResponse {
                error: self.to_string(),
                cause: self.source().unwrap().to_string(),
            })
    }
}

pub fn context_err(e: ClientHttpError, ctx: &str) -> ApiError {
    ApiError::new(e, ctx)
}
