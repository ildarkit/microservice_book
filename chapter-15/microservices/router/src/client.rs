use awc::Client;
use thiserror;
use awc::error::{SendRequestError, PayloadError, JsonPayloadError};
use serde::{Serialize, Deserialize};

#[derive(thiserror::Error, Debug)]
pub enum ClientHttpError {  
    #[error(transparent)]
    SendError(#[from] SendRequestError),
    #[error(transparent)]
    PayloadError(#[from] PayloadError),
    #[error(transparent)]
    JsonPayloadError(#[from] JsonPayloadError),
}

pub async fn get_request(url: &str) -> Result<Vec<u8>, ClientHttpError> {
    let res = Client::default().get(url)
        .send()
        .await
        .map_err(ClientHttpError::SendError)? 
        .body()
        .await
        .map_err(ClientHttpError::PayloadError)?;
    Ok(res.to_vec())
}

pub async fn post_request<T, O>(url: &str, params: T) -> Result<O, ClientHttpError>
where
    T: Serialize,
    O: for <'de> Deserialize<'de> + 'static,
{
    let res = Client::default().post(url)
        .send_form(&params)
        .await
        .map_err(ClientHttpError::SendError)?
        .json::<serde_json::Value>()
        .await
        .map_err(ClientHttpError::JsonPayloadError)?;
    Ok(serde_json::from_value(res).unwrap())
}

