use serde_derive::Serialize;
use crate::schema::users;
use diesel::prelude::*;

#[derive(Debug, Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str,
}
