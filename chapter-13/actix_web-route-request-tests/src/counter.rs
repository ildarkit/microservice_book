use log;
use std::fmt;
use futures::Future;
use std::cell::RefCell;

use crate::cache::CacheLink;
use crate::client::ClientHttpError;
use crate::LinksMap;

#[derive(Clone)]
pub struct CountState {
    counter: RefCell<i64>,
    cache: CacheLink,
    pub links: LinksMap,
}

impl CountState {
    pub fn new(cache: CacheLink, links: LinksMap) -> Self {
        Self {
            counter: RefCell::default(),
            cache,
            links,
        }
    }

    fn get_count(&self) -> i64 {
        *self.counter.borrow()
    }

    pub fn update_count(&self) {
        let value = self.get_count();
        *self.counter.borrow_mut() = value + 1;
    }

    pub async fn cache<F>(&self, path: &str, fut: F)
        -> Result<Vec<u8>, ClientHttpError>
        where
            F: Future<Output = Result<Vec<u8>, ClientHttpError>> + 'static,
    {
        let link = self.cache.clone();
        let path = path.to_owned();
        let res = link.get_value(&path).await
            .map_err(|e| {
                let ctx = "Failed to get from the cache";
                log ::warn!("{ctx}\n Caused by:\n\t{e}"); 
            })
            .ok();
        let res = match res {
            Some(Some(res)) => {
                log::debug!("Received cached response");
                res
            },
            Some(None) | None => {
                self.get_data(&path, fut).await?
            },
        };
        Ok(res)
    }
    
    async fn get_data<F>(&self, path: &str, fut: F) -> Result<Vec<u8>, ClientHttpError>
        where
            F: Future<Output = Result<Vec<u8>, ClientHttpError>> + 'static,
    {
        let data = fut.await
            .map_err(|e| {
                let ctx = "Failed to get response from service"; 
                log::error!("{ctx}\n Caused by:\n\t{e}");
                e
            })?;
        let link = self.cache.clone();
        link.set_value(path, &data).await
            .map_err(|e| {
                let ctx = "Failed to set to the cache";
                log::warn!("{ctx}\n Caused by:\n\t{e}");
            }).ok();
        Ok(data)
    }
}

impl fmt::Display for CountState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_count())
    }
}
