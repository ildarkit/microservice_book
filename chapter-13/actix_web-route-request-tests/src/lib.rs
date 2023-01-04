pub mod client;
pub mod handlers;
pub mod counter;
pub mod middleware;
pub mod error;
pub mod cache;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct LinksMap {
    pub signup: String,
    pub signin: String,
    pub new_comment: String,
    pub comments: String,
}
