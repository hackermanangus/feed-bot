use std::sync::Arc;

use regex::Regex;
use serenity::futures::{stream, StreamExt};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::utils::Colour;
use soup::{NodeExt, QueryBuilderExt};
use sqlx::{Error as SqlError,
           sqlite::SqlitePool,
};
use tokio::task;

use crate::structures::*;

/// Build client and fetch the page
async fn fetch(lk: &String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let result = client.post(lk)
        .send()
        .await?
        .text()
        .await?;
    Ok(result)
}

//noinspection DuplicatedCode
pub async fn retrieve_handle_channel(db: &SqlitePool, c_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel,
    "SELECT * FROM boxnovel WHERE channel_id=?", c_id)
        .fetch_all(&mut pool).await;
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(_) => return Err(format!("No novels are linked to this channel"))
    };
    // Blocking thread otherwise it will block the bot like the one under this
    // piece of code
    let novels: String = task::spawn_blocking(move || {
        cursor
            .iter()
            .map(|x| x.novel.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }).await.unwrap_or_else(|_| "".to_string());

    if novels == "" { return Err("No novels are linked to this channel".to_string()); }
    Ok(novels)
}

/// Handles the retrieving of all linked novels in the guild
pub async fn retrieve_handle_guild(db: &SqlitePool, g_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel, "SELECT * FROM boxnovel WHERE guild_id=?", g_id)
        .fetch_all(&mut pool).await;
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(_) => return Err(format!("No novels are linked to this guild"))
    };
    // Had to make it an blocking thread because I realised
    // that this would block
    let novels: String = task::spawn_blocking(move || {
        cursor
            .iter()
            .map(|x| x.novel.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }).await.unwrap_or_else(|_| "".to_string());

    if novels == "" { return Err("No novels are linked to this guild".to_string()); }
    Ok(novels)
}

//noinspection DuplicatedCode,DuplicatedCode
/// Should be called in an async loop to track the updates of a novel
/// TODO: Still working on this and it's not near done yet
/// TODO: We need to compare the two and then send a message
/// TODO: We have already retrieved the old and the new one, we just need to match them against each other
///
pub async fn check_updates_all(db: &SqlitePool, http: &Arc<Http>) -> Result<(), String> {
    // Retrieving ALL SQL ROWS here
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel,
    "SELECT * FROM boxnovel").fetch_all(&mut pool).await;
    // Checking if they're not empty
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(e) => return Err(e.to_string())
    };
    // Starting the process of updating the channels
    let mut stream = stream::iter(cursor);
    while let Some(boxnovel) = stream.next().await {
        // Getting the currently stored novels in an array
        let current = boxnovel.convert().await;
        //println!("{:?}", &current); DEBUG PURPOSES
        // Retrieve the possible updates html of the web page
        let html = fetch(&boxnovel.novel).await.unwrap();
        let new = task::spawn_blocking(move || {
            process_soup(html, boxnovel.novel, boxnovel.channel_id, boxnovel.guild_id)
        }).await;

        let novel = match new {
            Ok(ok) => match ok {
                Some(s) => s.convert().await,
                None => return Err("Unable to locate chapters".to_string()),
            }
            Err(_) => return Err("Unable to locate chapters".to_string())
        };
        let process_stream = stream::iter(novel.current.clone());
        let rev_true_chapters = process_stream
            .filter_map(|x| async {
                if !current.current.contains(&x) { Some(x) } else { None }
            })
            .collect::<Vec<String>>().await;
        let true_chapters = task::spawn_blocking(move || {
            rev_true_chapters.iter().rev().map(|x| x.to_string()).collect::<Vec<String>>()
        }).await.unwrap();
        let mut true_stream = stream::iter(true_chapters);
        while let Some(ch) = true_stream.next().await {
            let channel = ChannelId(current.c_id.parse::<u64>().unwrap());
            let _ = channel.send_message(http, |m| {
                m.embed(|e| {
                    e.title(format!("New Chapter for {}", &novel.title));
                    e.url(&ch);
                    e.description(&ch);
                    e.colour(Colour::DARK_GOLD)
                });
                m
            }).await;
            update_handle(db, novel.convert().await).await;
        }
    }
    Ok(())
}

///Update a row in the table
async fn update_handle(db: &SqlitePool, sqlbox: SQLResultBoxnovel) {
    let mut pool = db.acquire().await.unwrap();
    let _ = sqlx::query("UPDATE boxnovel SET
                current=? WHERE novel=?
        ")
        .bind(sqlbox.current)
        .bind(sqlbox.novel)
        .execute(&mut pool).await;
}

/// Delete a channel from the database so it no longer sends updates
pub async fn delete_handle_channel(db: &SqlitePool, link: String, c_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let query = sqlx::query("DELETE FROM boxnovel
        WHERE channel_id=? AND novel=?
    ")
        .bind(c_id.clone())
        .bind(link.clone())
        .execute(&mut pool).await;
    return match query {
        Ok(_) => Ok(format!("<{}> has been removed from <#{}>", link, c_id)),
        Err(_) => Err(format!("Failed to remove <{}> from <#{}>. Novel not found linked to this channel", link, c_id, ))
    };
}

/// Delete a novel from the entire guild so it no longer gets updates
pub async fn delete_handle_guild(db: &SqlitePool, link: String, g_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let query = sqlx::query("DELETE FROM boxnovel
        WHERE guild_id=? AND novel=?

    ")
        .bind(g_id.clone())
        .bind(link.clone())
        .execute(&mut pool).await;
    return match query {
        Ok(_) => Ok(format!("<{}> has been removed from from all channels", link)),
        Err(_) => Err(format!("Failed to remove <{}> from all channels. Novel not found linked to any channel", link))
    };
}

///noinspection DuplicateCode
/// Handles the insertion into the SQLite database
pub async fn initial_handle(db: &SqlitePool, link: String, c_id: String, g_id: String) -> Result<String, String> {

    // Every Err() is returning to the discord command where the message is sent to the user,
    let before = fetch(&link).await;
    let result = match before {
        Ok(x) => x,
        Err(e) => return Err(format!("Invalid link provided [{}]", e))
    };
    // Using tokio spawn_blocking because the soup crate is non-async
    let new_novel_unhandled = task::spawn_blocking(move || {
        process_soup(result, link, c_id, g_id)
    }).await;

    let sqlbox = match new_novel_unhandled {
        Ok(ok) => match ok {
            Some(s) => s,
            None => return Err("Unable to locate chapters".to_string()),
        }
        Err(_) => return Err("Unable to locate chapters".to_string())
    };

    let after = insert_into_db(db, sqlbox).await;
    return match after {
        Ok(_) => Ok("Success".to_string()),
        Err(e) => Err(e.to_string())
    };
}

/// Inserts a new novel into the SQLite database
async fn insert_into_db(db: &SqlitePool, sqlbox: SQLResultBoxnovel) -> Result<u64, SqlError> {
    let mut pool = db.acquire().await?;
    let query = sqlx::query("INSERT INTO boxnovel
                VALUES (?, ?, ?, ?, ?)
        ")
        .bind(sqlbox.guild_id)
        .bind(sqlbox.channel_id)
        .bind(sqlbox.title)
        .bind(sqlbox.novel)
        .bind(sqlbox.current)
        .execute(&mut pool).await;
    query
}

/// Gathers the information needed from the boxnovel html
fn process_soup(s: String, lk: String, c_id: String, g_id: String) -> Option<SQLResultBoxnovel> {
    let soup = soup::Soup::new(&s);
    let result = soup.tag("div")
        .attr("class", "post-title")
        .find();
    let title = match result {
        Some(x) => clear(x.text()),
        None => return None,
    };
    let ch_links: String = soup.tag("li")
        .attr("class", "wp-manga-chapter")
        .limit(30)
        .find_all()
        .filter_map(|r| r.tag("a").find())
        .map(|r| r.get("href").unwrap())
        .collect::<Vec<String>>()
        .join(" ");
    // let mut vec_chapters: Vec<Chapter> = vec![];
    // for (i, j) in ch_titles.iter().rev().zip(ch_links.iter().rev()) {
    //     vec_chapters.push(
    //         Chapter::new(i.to_string(), j.to_string())
    //     )
    // }

    Some(SQLResultBoxnovel {
        guild_id: g_id,
        channel_id: c_id,
        title,
        novel: lk,
        current: ch_links,
    })
}

/// REMOVES ALL THE WEIRD WHITESPACES, TABS, NEW LINES
fn clear(s: String) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]\s").unwrap();
    let s = s.replace("	", "");
    let s = s.replace("\n", "");
    let s = re.replace_all(&s, "").to_string();
    s
}
