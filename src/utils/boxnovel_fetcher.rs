use regex::Regex;
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
pub async fn retrieve_handle_channel(db: &SqlitePool, c_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel,
    "SELECT * FROM boxnovel WHERE channel_id=?", c_id)
        .fetch_all(&mut pool).await;
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(_) => return Err(format!("No novels are linked to this channel"))
    };
    let novels = cursor
        .iter()
        .map(|x| x.novel.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    if novels == "" { return Err("No novels are linked to this channel".to_string()) }
    Ok(novels)
}
pub async fn retrieve_handle_guild(db: &SqlitePool, g_id: String) -> Result<String, String> {
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel, "SELECT * FROM boxnovel WHERE guild_id=?", g_id)
        .fetch_all(&mut pool).await;
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(_) => return Err(format!("No novels are linked to this guild"))
    };
    let novels = cursor
        .iter()
        .map(|x| x.novel.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    if novels == "" { return Err("No novels are linked to this guild".to_string()) }
    Ok(novels)
}

pub async fn check_updates_all(db: &SqlitePool) -> Result<(), String> {
    let mut pool = db.acquire().await.unwrap();
    let cursor = sqlx::query_as!(SQLResultBoxnovel,
    "SELECT * FROM boxnovel").fetch_all(&mut pool).await;
    let cursor: Vec<SQLResultBoxnovel> = match cursor {
        Ok(inner) => inner,
        Err(e) => return Err(e.to_string())
    };

    println!("{:?}", &cursor[0].convert().await);

    Ok(())

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
        process_soup(result, link)
    }).await;
    let new_novel = match new_novel_unhandled {
        Ok(ok) => match ok {
            Some(s) => s,
            None => return Err("Unable to locate chapters".to_string()),
        }
        Err(_) => return Err("Unable to locate chapters".to_string())
    };

    let after = insert_into_db(db, c_id, g_id, new_novel).await;
    return match after {
        Ok(_) => Ok("Success".to_string()),
        Err(e) => Err(e.to_string())
    };
}

/// Inserts a new novel into the SQLite database
async fn insert_into_db(db: &SqlitePool, c_id: String, g_id: String, n: Novel) -> Result<u64, SqlError> {
    let mut pool = db.acquire().await?;
    let query = sqlx::query("INSERT INTO boxnovel
                VALUES (?, ?, ?, ?)
        ")
        .bind(g_id)
        .bind(c_id)
        .bind(n.link)
        .bind(n.chapters
            .iter()
            .map(|s| format!("{} ", s.link))
            .collect::<String>()
        ).execute(&mut pool).await;

    query
}

/// Gathers the information needed from the boxnovel html
fn process_soup(s: String, lk: String) -> Option<Novel> {
    let soup = soup::Soup::new(&s);
    let result = soup.tag("div")
        .attr("class", "post-title")
        .find();
    let title = match result {
        Some(x) => clear(x.text()),
        None => return None,
    };
    let ch_titles: Vec<String> = soup.tag("li")
        .attr("class", "wp-manga-chapter")
        .limit(30)
        .find_all()
        .filter_map(|r| r.tag("a").find())
        .map(|r| clear(r.text()))
        .collect::<Vec<String>>();
    let ch_links: Vec<String> = soup.tag("li")
        .attr("class", "wp-manga-chapter")
        .limit(30)
        .find_all()
        .filter_map(|r| r.tag("a").find())
        .map(|r| r.get("href").unwrap())
        .collect::<Vec<String>>();
    // let mut vec_chapters: Vec<Chapter> = vec![];
    // for (i, j) in ch_titles.iter().rev().zip(ch_links.iter().rev()) {
    //     vec_chapters.push(
    //         Chapter::new(i.to_string(), j.to_string())
    //     )
    // }

    // Opted to map instead of a for loop for a more concise way to see what's happening
    // Also slightly more efficient I think
    let vec_chapters: Vec<Chapter> = ch_titles
        .iter().rev()
        .zip(ch_links.iter().rev())
        .map(|(title, link)| Chapter::new(title.to_string(), link.to_string()))
        .collect::<Vec<Chapter>>();

    Some(Novel::new(title, lk, vec_chapters))
}

/// REMOVES ALL THE WEIRD WHITESPACES, TABS, NEW LINES
fn clear(s: String) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]\s").unwrap();
    let s = s.replace("	", "");
    let s = s.replace("\n", "");
    let s = re.replace_all(&s, "").to_string();
    s
}
