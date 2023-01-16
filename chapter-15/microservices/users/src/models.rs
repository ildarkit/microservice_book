use serde_derive::Serialize;
use diesel::prelude::*;
use super::schema::users;

#[derive(Debug, Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}
