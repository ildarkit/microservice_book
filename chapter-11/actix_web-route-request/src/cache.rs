use log::error;
use thiserror;
use anyhow::{self, Context};
use actix::prelude::*;
use redis::{Commands, Client};

pub struct CacheActor {
    client: Client,
    expiration: usize,
}

impl CacheActor {
    pub fn new(addr: &str, expiration: usize) -> Self {
        let client = Client::open(addr).unwrap();
        Self { client, expiration }
    }
}

impl Actor for CacheActor {
    type Context = SyncContext<Self>;
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {  
    #[error(transparent)]
    MailboxError(#[from] MailboxError),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

struct SetValue {
    pub path: String,
    pub content: Vec<u8>,
}

impl Message for SetValue {
    type Result = Result<(), CacheError>;
}

impl Handler<SetValue> for CacheActor {
    type Result = Result<(), CacheError>;

    fn handle(&mut self, msg: SetValue, _: &mut Self::Context) 
        -> Self::Result
    {
        self.client.get_connection()
            .context("Failed to connect to Redis")?
            .set_ex(msg.path, msg.content, self.expiration)
            .context("Failed to set to cache")?;
        Ok(())
    }
}

struct GetValue {
    pub path: String,
}

impl Message for GetValue {
    type Result = Result<Option<Vec<u8>>, CacheError>;
}

impl Handler<GetValue> for CacheActor {
    type Result = Result<Option<Vec<u8>>, CacheError>;

    fn handle(&mut self, msg: GetValue, _: &mut Self::Context)
        -> Self::Result 
    {
        let value = self.client.get_connection()
            .context("Failed to connect to Redis")?
            .get(&msg.path)
            .context("Failed to get from cache")?;
        Ok(value)
    }
}

#[derive(Clone)]
pub struct CacheLink {
    addr: Addr<CacheActor>,
}

impl CacheLink { 
    pub fn new(addr: Addr<CacheActor>) -> Self {
        Self { addr }
    }

    pub async fn get_value(&self, path: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let msg = GetValue {
            path: path.to_owned(),
        };
        self.addr.send(msg).await
            .map_err(|e| { 
                error!("{}", e);
                e
            })?
    }

    pub async fn set_value(&self, path: &str, value: &[u8]) -> Result<(), CacheError> {
        let msg = SetValue {
            path: path.to_owned(),
            content: value.to_owned(),
        };
        self.addr.send(msg).await
            .map_err(|e| {
                error!("{}", e);
                e
            })?
    }
}
