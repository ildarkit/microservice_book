extern crate clap;
extern crate postgres;

use clap::{arg, command, Command};
use postgres::{Client, error::Error, config::SslMode};

fn create_table(conn: &Client) -> Result<(), Error> {
    conn.execute("CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    )", &[])
        .map(drop)
}

fn create_user(conn: &Client, name: &str, email: &str) -> Result<(), Error> {
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)",
    &[&name, &email])
        .map(drop)
}

fn list_users(conn: &Client,) -> Result<Vec<(String, String)>, Error> {
    let res = conn.query("SELECT name, email FROM users", &[])?.into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();
    Ok(res)
}

fn main() {
   unimplemented!(); 
}
