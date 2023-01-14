use log;

use crate::cache_actor::CacheLink;
use crate::client::{self, ClientHttpError};

pub async fn cache(
    cache: &CacheLink,
    cache_key: &str,
    client_addr: &str
)
    -> Result<Vec<u8>, ClientHttpError>
{
    let res = cache.get_value(&cache_key).await;
    let res =
        if res.is_err() {
            res.map_err(|e| {
                let ctx = "Failed to get from the cache";
                log::warn!("{ctx}\n Caused by:\n\t{e}"); 
            }).ok();
            let data = client::get_request(&client_addr).await
                .map_err(|e| {
                    let ctx = "Failed to get response from service"; 
                    log::error!("{ctx}\n Caused by:\n\t{e}");
                    e
                })?;
            cache.set_value(&cache_key, &data).await
                .map_err(|e| {
                    let ctx = "Failed to set to the cache";
                    log::warn!("{ctx}\n Caused by:\n\t{e}");
                }).ok();
            data.to_vec()
        } else {
            log::debug!("Received cached response");
            res.unwrap().unwrap()
        };
    Ok(res)
}
