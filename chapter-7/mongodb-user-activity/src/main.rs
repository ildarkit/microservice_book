use std::error::Error;
use std::time::Duration;
use chrono::offset::Utc;
use serde_derive::Deserialize;
use clap::{arg, command, Command};
use mongodb::bson;
use mongodb::options::ClientOptions;
use mongodb::sync::{Client, Database};
use mongodb::error::Error as MongodbError;

const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";

#[derive(Deserialize, Debug)]
struct Activity {
    user_id: String,
    activity: String,
    datetime: String,
}

fn add_activity(conn: &Database, activity: Activity) -> Result<(), MongodbError> {
    let doc = bson::doc! {
        "user_id": activity.user_id,
        "activity": activity.activity,
        "datetime": activity.datetime,
    };
    let coll = conn.collection("activities");
    coll.insert_one(doc, None).map(drop)
}

fn list_activities(conn: &Database) -> Result<Vec<Activity>, MongodbError> {
    conn.collection("activities").find(None, None)?
        .try_fold(Vec::new(), |mut vec, doc| {
            let doc = doc?;
            let activity: Activity =
                bson::from_bson(bson::Bson::Document(doc))?;
            vec.push(activity);
            Ok(vec)
        })
}

fn main() -> Result<(), Box<dyn Error>> {
   let matches = command!()
       .subcommand_required(true)
       .arg(arg!(database: -d --db <ADDR> "Sets an address of db connection"))
       .subcommand(Command::new(CMD_LIST)
                   .about("print activities list of users"))
       .subcommand(Command::new(CMD_ADD)
                   .about("add user to the table")
                   .arg(arg!(USER_ID: "Sets the id of a user")
                        .required(true))
                   .arg(arg!(ACTIVITY: "Set the activity of a user")
                        .required(true)))
       .get_matches();

   let default_addr = "mongodb://localhost:27017/admin".to_string();
   let addr = matches.get_one::<String>("database")
       .unwrap_or(&default_addr);
   let mut opts = ClientOptions::parse(addr)?;
   if let None = opts.max_pool_size { 
       opts.max_pool_size = Some(4);
   }
   if let None = opts.connect_timeout {
       opts.connect_timeout = Some(Duration::from_secs(10));
   }
   let client = Client::with_options(opts)?;
   let conn = client.database("user_activity");

   match matches.subcommand() {
       Some((CMD_ADD, activity_matches)) => {
           let user_id = activity_matches
               .get_one::<String>("USER_ID").unwrap().to_owned();
           let activity = activity_matches
               .get_one::<String>("ACTIVITY").unwrap().to_owned();
           let activity = Activity {
               user_id,
               activity,
               datetime: Utc::now().to_string(),
           };
           add_activity(&conn, activity)?;
       },
       Some((CMD_LIST, _)) => {
           let list = list_activities(&conn)?;
           for item in list {
               println!("User: {:20} Activity: {:20} DateTime: {:20}",
                        item.user_id, item.activity, item.datetime);
           }
       },
       _ => { },
   }
   Ok(())
}
