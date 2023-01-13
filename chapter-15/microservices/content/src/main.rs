#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

mod comment;
mod schema;

use log::error;
use rocket::{Build, Rocket};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket_sync_db_pools::{
    database,
    diesel::PgConnection
};
use comment::{Comment, NewComment};

#[database("postgres_database")]
pub struct Db(PgConnection);

#[get("/comments")]
async fn list(conn: Db) -> Json<Vec<Comment>> {
    conn.run(|c| Json(Comment::all(&c))).await
}

#[post("/new_comment", data = "<comment_form>")]
async fn add_new(comment_form: Form<NewComment>, conn: Db) {
    let comment = comment_form.into_inner();
    conn.run(|c| Comment::insert(comment, &c)).await;
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .mount("/", routes![list, add_new])
}
