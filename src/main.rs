use std::env;

use dotenv;
use log::error;
use serenity::{
    async_trait,
    prelude::*,
};
use serenity::framework::standard::{
    CommandResult,
    macros::{
        command,
        group,
    },
    StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::prelude::Ready;
use serenity::prelude::{Context, EventHandler};
use sqlx::{Connect, SqliteConnection, SqlitePool};

use crate::db::{database_connect, initialise_database_tables};

mod db;

struct Db;

impl TypeMapKey for Db {
    type Value = SqlitePool;
}


#[group]
#[commands(ping)]
struct General;

// #[group]
// #[prefixes("bn", "boxnovel")]
// #[commands(connect)]
// struct Boxnovel;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = env::var("TOKEN")
        .expect("Expected a token. None found");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("f!"))
        .group(&GENERAL_GROUP);
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let mut db = match database_connect().await {
        Ok(d) => d,
        Err(e) => {
            panic!("Couldn't connect database {}", e);
        }
    };
    if let Err(e) = initialise_database_tables(&mut db.acquire().await.unwrap()).await {
        panic!("Couldn't setup table {}", e);
    }
    if let Err(why) = client.start().await {
        println!("Err with client: {:?}", why);
    }

    {
        let mut data = client.data.write().await;
        data.insert::<Db>(db);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong").await?;

    Ok(())
}