use std::error::Error;
use std::ops::DerefMut;
use clap::{arg, command, Command};
use r2d2_redis::redis::{Commands, Connection, RedisError};
use r2d2_redis::RedisConnectionManager;
use std::collections::HashMap;

const SESSIONS: &str = "sessions";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const CMD_REMOVE: &str = "remove";

fn add_session(conn: &mut Connection, token: &str, uid: &str) -> Result<(), RedisError> {
    conn.hset(SESSIONS, token, uid)
}

fn remove_session(conn: &mut Connection, token: &str) -> Result<(), RedisError> {
    conn.hdel(SESSIONS, token)
}

fn list_session(conn: &mut Connection) -> Result<HashMap<String, String>, RedisError> {
    conn.hgetall(SESSIONS)
}

fn main() -> Result<(), Box<dyn Error>> {
   let matches = command!()
       .subcommand_required(true)
       .arg(arg!(database: -d --db <ADDR> "Sets an address of db connection"))
       .subcommand(Command::new(CMD_LIST)
                   .about("print list of sessions"))
       .subcommand(Command::new(CMD_REMOVE)
                   .about("remove a session")
                   .arg(arg!(TOKEN: "Sets the token of a user")
                        .required(true)))
       .subcommand(Command::new(CMD_ADD)
                   .about("add a session")
                   .arg(arg!(TOKEN: "Set the token of a user")
                        .required(true))
                   .arg(arg!(UID: "Set the uid of a user")
                        .required(true)))
       .get_matches();

   let default_addr = "redis://127.0.0.1/".to_string();
   let addr = matches.get_one::<String>("database")
       .unwrap_or(&default_addr);
   let manager = RedisConnectionManager::new(&**addr)?;
   let pool = r2d2::Pool::new(manager).unwrap();
   let mut conn = pool.get()?;
   let mut conn = conn.deref_mut();

   match matches.subcommand() {
       Some((CMD_ADD, sess_matches)) => {
           let token = sess_matches.get_one::<String>("TOKEN").unwrap().to_owned();
           let uid = sess_matches.get_one::<String>("UID").unwrap().to_owned();
           add_session(&mut conn, &token, &uid)?;
       },
       Some((CMD_LIST, _)) => {
           println!("LIST");
           let sessions = list_session(&mut conn)?;
           for (token, uid) in sessions {
               println!("Token {:20}     Uid {:20}", token, uid);
           }
       },
       Some((CMD_REMOVE, sess_matches)) => {
           let token = sess_matches.get_one::<String>("TOKEN").unwrap().to_owned();
           remove_session(&mut conn, &token)?;
       },
       _ => { },
   }
   Ok(())
}
