#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod comment;
mod schema;

use log::error;
use rocket::{Build, Rocket};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket_sync_db_pools::{
    database,
    diesel::SqliteConnection
};
use comment::{Comment, NewComment};

#[database("sqlite_database")]
pub struct Db(SqliteConnection);

embed_migrations!();

#[get("/comments")]
async fn list(conn: Db) -> Json<Vec<Comment>> {
    conn.run(|c| Json(Comment::all(&c))).await
}

#[post("/new_comment", data = "<comment_form>")]
async fn add_new(comment_form: Form<NewComment>, conn: Db) {
    let comment = comment_form.into_inner();
    conn.run(|c| Comment::insert(comment, &c)).await;
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let db = Db::get_one(&rocket)
        .await
        .expect("no database connection");
    match db.run(|conn| embedded_migrations::run(&*conn)).await {
        Ok(_) => rocket,
        Err(err) => {
            error!("Failed to run database migrations: {:?}", err);
            rocket
        },
    }
}

/*  for diesel_migrations 2.0

    use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    fn migrate(conn: &mut impl MigrationHarness<SqliteConnection>)
        -> Result<(), Box<dyn Error + Send + Sync + 'static>>
    {
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    }
*/

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Database Migrations", run_migrations))
        .mount("/", routes![list, add_new])
}
