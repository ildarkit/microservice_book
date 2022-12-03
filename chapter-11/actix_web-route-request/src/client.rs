use awc::Client;
use actix_web::web::Bytes;
use serde::{Serialize, Deserialize};

use crate::error::ClientError;

pub async fn get_request(url: &str) -> Result<Bytes, ClientError> {
    let res = Client::default().get(url)
        .send()
        .await?
        .body()
        .await?;
    Ok(res)
}

pub async fn post_request<T, O>(url: &str, params: T) -> Result<O, ClientError>
where
    T: Serialize,
    O: for <'de> Deserialize<'de> + 'static,
{
    let res = Client::default().post(url)
        .send_form(&params)
        .await?
        .json::<O>()
        .await?;
    Ok(res)
}

