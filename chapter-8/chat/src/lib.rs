mod models;
mod schema;

use std::env;
use diesel::prelude::*;
use chrono::Utc;
use failure::{Error, format_err};
use self::models::{Channel, Id, Membership, Message, User};
use self::schema::{channels, memberships, messages, users};
use diesel::{r2d2::{Pool, ConnectionManager, PooledConnection}, insert_into};

pub type PoolConnection = Pool<ConnectionManager<PgConnection>>;
pub type PooledConn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct Api
{
    pool: PoolConnection,
}

impl Api {
    pub fn connect() -> Result<Self, Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or("postgres://postgres@localhost:5432".to_string());
        let manager = ConnectionManager::new(database_url);
        let pool = Pool::builder().build(manager)?; 
        Ok(Self { pool })
    }

    pub fn get_connect(&self)
        -> Result<PooledConn, Error>
    {
        self.pool.get().map_err(Error::from)
    }

    pub fn register_user(&self, conn: &mut PooledConn, email: &str)
        -> Result<User, Error>
    {
        insert_into(users::table)
            .values((users::email.eq(email),))
            .get_result(conn)
            .map_err(Error::from)
    }

    pub fn create_channel(
        &self, conn: &mut PooledConn, user_id: Id, title: &str, is_public: bool
        )
        -> Result<Channel, Error>
    {
        conn.transaction(|conn| {
            let channel: Channel = insert_into(channels::table)
                .values((
                        channels::user_id.eq(user_id),
                        channels::title.eq(title),
                        channels::is_public.eq(is_public),
                        ))
                .get_result(conn)
                .map_err(Error::from)?;
            self.add_member(conn, channel.id, user_id)?;
            Ok(channel)
        })
    }

    pub fn publish_channel(&self, conn: &mut PooledConn, channel_id: Id)
        -> Result<(), Error>
    {
        let channel = channels::table
            .filter(channels::id.eq(channel_id))
            .select((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    channels::created_at,
                    channels::updated_at,
                    ))
            .first::<Channel>(conn)
            .optional()
            .map_err(Error::from)?;
        if let Some(channel) = channel {
            diesel::update(&channel)
                .set(channels::is_public.eq(true))
                .execute(conn)?;
            Ok(())
        } else {
            Err(format_err!("channel not found"))
        }
    }

    pub fn add_member(&self, conn: &mut PooledConn, channel_id: Id, user_id: Id) 
        -> Result<Membership, Error>
    {
        insert_into(memberships::table)
            .values((
                    memberships::channel_id.eq(channel_id),
                    memberships::user_id.eq(user_id),
                    ))
            .get_result(conn)
            .map_err(Error::from)
    }

    pub fn add_message(&self, conn: &mut PooledConn, channel_id: Id, user_id: Id, text: &str)
        -> Result<Message, Error> 
    {
        let ts_now = Utc::now().naive_utc();
        insert_into(messages::table)
            .values((
                    messages::timestamp.eq(ts_now),
                    messages::channel_id.eq(channel_id),
                    messages::user_id.eq(user_id),
                    messages::text.eq(text)
                    ))
            .get_result(conn)
            .map_err(Error::from)
    }

    pub fn delete_message(&self, conn: &mut PooledConn, message_id: Id)
        -> Result<(), Error>
    {
        diesel::delete(messages::table)
            .filter(messages::id.eq(message_id))
            .execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Api;

    #[test]
    fn create_users() {
        let api = Api::connect().unwrap();
        let conn = &mut api.get_connect().unwrap();
        let user_1 = api.register_user(conn, "user_1@example.com").unwrap();
        let user_2 = api.register_user(conn, "user_2@example.com").unwrap();
        let channel = api.create_channel(conn, user_1.id, "My Channel", false).unwrap();
        api.publish_channel(conn, channel.id).unwrap();
        api.add_member(conn, channel.id, user_2.id).unwrap();
        let message = api.add_message(conn, channel.id, user_1.id, "Welcome!").unwrap();
        api.add_message(conn, channel.id, user_2.id, "Hi!").unwrap();
        api.delete_message(conn, message.id).unwrap();
    }
}
