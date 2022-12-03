use std::fmt;
use std::error::Error;
use thiserror;
use anyhow;
use serde::Serialize;
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use awc::error::{SendRequestError, PayloadError, JsonPayloadError};

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
    cause: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Client request fail")]
    SendError { source: SendRequestError },

    #[error(transparent)]
    ResponseError(#[from] PayloadError),

    #[error(transparent)]
    JsonResponseError(#[from] JsonPayloadError),

    #[error("Other error")]
    ActixError
}

impl From<SendRequestError> for ClientError {
    fn from(error: SendRequestError) -> ClientError {
        ClientError::SendError { source: error }
    }
}

#[derive(Debug)]
pub struct ApiError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub err_type: ClientError,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.err_type {
            ClientError::SendError { source: SendRequestError::Url(_) } => {
                StatusCode::BAD_REQUEST
            },
            _ => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(ApiErrorResponse {
                error: self.message(),
                cause: self.cause(),
            })
    }
}

impl From<ClientError> for ApiError {
    fn from(error: ClientError) -> ApiError {
        ApiError {
            message: Some(error.to_string()),
            cause: Some(
                format!("{}", error.source().unwrap())
            ),
            err_type: error,
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> ApiError {
        ApiError {
            message: Some(error.to_string()),
            cause: Some(
                format!("{}", error.source().unwrap())
            ),
            err_type: ClientError::ActixError,
        }
    }
}

impl ApiError {
    fn message(&self) -> String {
        self.to_string(&self.message)
    }

    fn cause(&self) -> String {
        self.to_string(&self.cause)
    }

    fn to_string(&self, value: &Option<String>) -> String {
        if let Some(value) = value {
            value.to_string()
        } else {
            "An unexpected error has occured".to_string()
        }
    }
}
