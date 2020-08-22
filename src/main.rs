use std::env;

use dotenv;
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
use sqlx::SqlitePool;

use crate::commands::{
    boxnovel::*
};
use crate::db::{database_connect, initialise_database_tables};

pub mod structures;

mod db;
mod utils;
mod commands;

struct Db;

impl TypeMapKey for Db {
    type Value = SqlitePool;
}


#[group]
#[commands(ping)]
struct General;

#[group]
#[prefixes(bn, boxnovel)]
#[commands(add, remove, test, check)]
struct Boxnovel;

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
        .configure(|c| c
            .prefix("f!")
            .case_insensitivity(true))
        .group(&GENERAL_GROUP)
        .group(&BOXNOVEL_GROUP);
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let db = match database_connect().await {
        Ok(d) => d,
        Err(e) => {
            panic!("Couldn't connect database {}", e);
        }
    };
    if let Err(e) = initialise_database_tables(&mut db.acquire().await.unwrap()).await {
        panic!("Couldn't setup table {}", e);
    }
    {
        let mut data = client.data.write().await;
        data.insert::<Db>(db);
    }
    if let Err(why) = client.start().await {
        println!("Err with client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong").await?;

    Ok(())
}