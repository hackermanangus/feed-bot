
use serenity::{
    prelude::*,
    async_trait,
};
use serenity::prelude::{EventHandler, Context};
use serenity::model::prelude::Ready;

use log::{debug, error, info};
use dotenv;
use std::env;
use serenity::model::channel::Message;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

struct Handler;

#[group]
#[commands(ping)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("f!"))
        .group(&GENERAL_GROUP);
    let token = env::var("TOKEN")
        .expect("Expected a token. None found");

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong").await?;

    Ok(())
}