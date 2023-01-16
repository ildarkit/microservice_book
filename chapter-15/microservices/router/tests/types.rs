#![allow(dead_code)]

use serde_derive::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UserId {
    id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub id: Option<i32>,
    pub uid: String,
    pub text: String,
}
