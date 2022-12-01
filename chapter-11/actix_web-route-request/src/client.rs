use awc::{Client, error::{SendRequestError, PayloadError, JsonPayloadError}};
use actix_web::{Responder, web::Bytes};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use futures::{future::Future, TryFutureExt, TryStreamExt, StreamExt};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Client request fail")]
    SendError { source: SendRequestError },

    #[error(transparent)]
    ResponseError(#[from] PayloadError),

    #[error(transparent)]
    JsonResponseError(#[from] JsonPayloadError)
}

pub async fn get_request(url: &str) -> Result<Bytes, ClientError> {
    Client::default().get(url)
        .send()
        .await
        .map_err(|err| ClientError::SendError {source: err})? 
        .body()
        .await
        .map_err(ClientError::ResponseError)
}

pub async fn post_request<T, O>(url: &str, params: T) -> Result<O, ClientError>
where
    T: Serialize,
    O: for <'de> Deserialize<'de> + 'static,
{
    Client::default().post(url)
        .send_form(&params)
        .await
        .map_err(|err| ClientError::SendError {source: err})?
        .json::<O>()
        .await
        .map_err(ClientError::JsonResponseError)
}

