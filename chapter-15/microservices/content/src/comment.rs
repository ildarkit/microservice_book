use diesel::{self, prelude::*};
use serde_derive::Serialize;
use rocket::form::FromForm;
use rocket_sync_db_pools::diesel::PgConnection;
use super::schema::comments;
use super::schema::comments::dsl::{comments as all_comments};

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[table_name = "comments"]
pub struct Comment {
    pub id: Option<i32>,
    pub uid: String,
    pub text: String,
}

#[derive(FromForm)]
pub struct NewComment {
    pub uid: String,
    pub text: String,
}

impl Comment {
    pub fn all(conn: &PgConnection) -> Vec<Comment> {
        all_comments.order(comments::id.desc()).load::<Comment>(conn).unwrap()
    }

    pub fn insert(comment: NewComment, conn: &PgConnection) -> bool {
        let t = Comment {
            id: None,
            uid: comment.uid,
            text: comment.text,
        };
        diesel::insert_into(comments::table)
            .values(&t)
            .execute(conn)
            .is_ok()
    }
}
