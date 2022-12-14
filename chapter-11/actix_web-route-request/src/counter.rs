use log;
use std::fmt;
use futures::Future;
use std::cell::RefCell;

use crate::cache::CacheLink;
use crate::client::ClientHttpError;

#[derive(Clone)]
pub struct CountState {
    counter: RefCell<i64>,
    cache: CacheLink,
}

impl CountState {
    pub fn new(cache: CacheLink) -> Self {
        Self {
            counter: RefCell::default(),
            cache,
        }
    }

    pub fn get_count(&self) -> i64 {
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
        let res = link.get_value(&path).await;
        let res =
            if res.is_err() {
                res.map_err(|e| {
                    let ctx = "An unexpected error occured when getting from the cache";
                    log::warn!("{ctx}\n Caused by:\n\t{e}"); 
                }).ok();
                let data = fut.await?;
                link.set_value(&path, &data).await
                    .map_err(|e| {
                        let ctx = "An unexpected error occured while storing to the cache";
                        log::warn!("{ctx}\n Caused by:\n\t{e}");
                    }).ok();
                data.to_vec()
            } else {
                res.unwrap().unwrap()
            };
        Ok(res)
    }
}

impl fmt::Display for CountState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_count())
    }
}
