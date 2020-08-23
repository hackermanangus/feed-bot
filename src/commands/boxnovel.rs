use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::framework::standard::Args;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

use crate::Db;
use crate::utils::{
    boxnovel_fetcher::*,
};

#[command]
#[aliases(rem, del, delete)]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // This command is used when a user wants to delete a novel from a channel
    // TODO: Check if novel exists first before trying to delete, so that the user does not get confused if the novel exists
    // TODO: Because of the way SQL Delete statements work
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let guild_id = || {
        let g_id = msg.guild_id;
        let guild_id = match g_id {
            Some(x) => x.to_string(),
            None => 0.to_string()
        };
        guild_id
    };

    let channel_id = || msg.channel_id.to_string();
    let novel = args.single::<String>()?;
    let flag = args.single::<String>().unwrap_or_else(|_| "default".to_string());
    let result = match flag.as_str() {
        // Checking if it's a DM channel
        "-g" if guild_id() != "0" => delete_handle_guild(db, novel, guild_id()).await,
        _ => delete_handle_channel(db, novel, channel_id()).await
    };

    match result {
        Ok(x) => msg.channel_id.say(&ctx.http, x).await?,
        Err(e) => msg.channel_id.say(&ctx.http, e).await?
    };

    Ok(())
}

#[command]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let g_id = msg.guild_id;
    // So if it's a DM channel, the bot won't crash, set the guild_id to 0
    // and in the delete function we just check if it's 0 and then force call
    // delete_handle_channel() instead of it's guild variant
    let guild_id = match g_id {
        Some(x) => x.to_string(),
        None => 0.to_string()
    };
    let channel_id = msg.channel_id.to_string();
    let novel: String = args.single::<String>()?;
    let result = initial_handle(db, novel, channel_id, guild_id).await;
    match result {
        Ok(x) => msg.channel_id.say(&ctx.http, x).await?,
        Err(e) => msg.channel_id.say(&ctx.http, e).await?
    };

    Ok(())
}

#[command]
async fn check(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let guild_id = || {
        let g_id = msg.guild_id;
        let guild_id = match g_id {
            Some(x) => x.to_string(),
            None => 0.to_string()
        };
        guild_id
    };

    let channel_id = || msg.channel_id.to_string();
    let flag = args.single::<String>().unwrap_or_else(|_| "default".to_string());
    let result = match flag.as_str() {
        // Checking if it's a DM channel
        "-g" if guild_id() != "0" => retrieve_handle_guild(db, guild_id()).await,
        _ => retrieve_handle_channel(db, channel_id()).await
    };

    match result {
        Ok(x) => msg.channel_id.say(&ctx.http, x).await?,
        Err(e) => msg.channel_id.say(&ctx.http, e).await?
    };

    Ok(())
}

#[command]
async fn test(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

