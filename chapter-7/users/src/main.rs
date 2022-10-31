extern crate clap;
extern crate postgres;

use clap::{arg, command, Command};
use postgres::{Client, error::Error, NoTls};
use serde_derive::Deserialize;

const CMD_CREATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const CMD_IMPORT: &str = "import";

#[derive(Deserialize, Debug)]
struct User {
    name: String,
    email: String,
}

fn create_table(conn: &mut Client) -> Result<(), Error> {
    conn.execute("CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        email VARCHAR NOT NULL)", &[])
        .map(drop)
}

fn create_user(conn: &mut Client, user: &User) -> Result<(), Error> {
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)",
    &[&user.name, &user.email])
        .map(drop)
}

fn list_users(conn: &mut Client,) -> Result<Vec<User>, Error> {
    let res = conn.query("SELECT name, email FROM users", &[])?.into_iter()
        .map(|row| {
            User {
                name: row.get(0),
                email: row.get(1)
            }
        })
        .collect();
    Ok(res)
}

fn main() -> Result<(), Error> {
   let matches = command!()
       .arg(arg!(database: -d --db <ADDR> "Sets an address of db connection"))
       .subcommand(Command::new(CMD_CREATE)
                   .about("create users table"))
       .subcommand(Command::new(CMD_IMPORT)
                   .about("import users from csv"))
       .subcommand(Command::new(CMD_ADD)
                   .about("add user to the table")
                   .arg(arg!(NAME: "Set the name of a user")
                        .required(true))
                   .arg(arg!(EMAIL: "Set the email of a user")
                        .required(true)))
       .subcommand(Command::new(CMD_LIST)
                   .about("print list of users"))
       .get_matches();

   let default_addr = "postgres://postgres@localhost:5432".to_string();
   let addr = matches.get_one::<String>("database")
       .unwrap_or(&default_addr);
   let mut conn = Client::connect(addr, NoTls)?;

   match matches.subcommand() {
       Some((CMD_CREATE, _)) => {
           create_table(&mut conn)?;
       },
       Some((CMD_ADD, user_matches)) => {
           let name = user_matches.get_one::<String>("NAME").unwrap().to_owned();
           let email = user_matches.get_one::<String>("EMAIL").unwrap().to_owned();
           let user = User {name, email};
           create_user(&mut conn, &user)?;
       },
       Some((CMD_LIST, _)) => {
           let list = list_users(&mut conn)?;
           for user in list {
               println!("Name {:20}     Email {:20}", user.name, user.email);
           }
       },
       _ => { },
   }
   Ok(())
}
