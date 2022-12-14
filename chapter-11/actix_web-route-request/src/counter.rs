use std::fmt;
use futures::Future;
use std::cell::RefCell;

use crate::cache::CacheLink;
use crate::client::ClientHttpError;

fn boxed<I, E, F>(fut: F) -> Box<dyn Future<Output = I>>
    where
        F: Future<Output = I> + 'static,
{
    Box::new(fut)
}

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
        let res = match link.get_value(&path).await.unwrap() {
            Some(cached) => cached,
            None => {
                let data = fut.await?;
                link.set_value(&path, &data).await;
                data.to_vec()
            },
        };
        Ok(res)
    }
}

impl fmt::Display for CountState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_count())
    }
}
