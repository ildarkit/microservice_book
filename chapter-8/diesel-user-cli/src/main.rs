use std::error::Error;
use uuid::Uuid;
use clap::{arg, command, Command};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
//use failure::Error as FailureError;
use diesel_user_cli::schema::users::{self, dsl::*};
use diesel_user_cli::models;

const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";

fn main() -> Result<(), Box<dyn Error>> {
   let matches = command!()
       .subcommand_required(true)
       .arg(arg!(database: -d --db <FILE> "Sets a file name of database"))
       .subcommand(Command::new(CMD_LIST)
                   .about("print a list with users")) 
       .subcommand(Command::new(CMD_ADD)
                   .about("add user to the table")
                   .arg(arg!(NAME: "Set the name of a user")
                        .required(true))
                   .arg(arg!(EMAIL: "Set the email of a user")
                        .required(true)))
       .get_matches();

   let default_path = "test.db".to_string();
   let path = matches.get_one::<String>("database")
       .unwrap_or(&default_path);
   let manager = ConnectionManager::<SqliteConnection>::new(path);
   let pool = r2d2::Pool::new(manager)?;
   let mut conn = pool.get()?;

   match matches.subcommand() {
       Some((CMD_ADD, user_matches)) => {
           let name_arg = user_matches.get_one::<String>("NAME").unwrap().to_string();
           let email_arg = user_matches.get_one::<String>("EMAIL").unwrap().to_string();
           let uuid = format!("{}", Uuid::new_v4());
           let new_user = models::NewUser {
               id: &uuid,
               name: &name_arg,
               email: &email_arg,
           };
           diesel::insert_into(users::table)
               .values(&new_user)
               .execute(&mut conn)
               .expect("Error saving a new user");
       },
       Some((CMD_LIST, _)) => {
           let items = users
               .load::<models::User>(&mut conn)?;
           for user in items {
               println!("{:?}", user);
           }
       },
       _ => { },
   }
   Ok(())
}
