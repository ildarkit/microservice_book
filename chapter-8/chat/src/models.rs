use chrono::NaiveDateTime;
use serde_derive::{Serialize, Deserialize};
use crate::schema::{users, channels, memberships, messages};
use diesel::prelude::*;

pub type Id = i32;

#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Id,
    pub email: String,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[diesel(table_name = channels)]
#[diesel(belongs_to(User))]
pub struct Channel {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[diesel(table_name = memberships)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Channel))]
pub struct Membership {
    pub id: Id,
    pub channel_id: Id,
    pub user_id: Id,
}

#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[diesel(table_name = messages)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Channel))]
pub struct Message {
    pub id: Id,
    pub timestamp: NaiveDateTime,
    pub channel_id: Id,
    pub user_id: Id,
    pub text: String,
}

