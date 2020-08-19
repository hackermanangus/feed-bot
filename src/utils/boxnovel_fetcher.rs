pub mod boxnovel_fetcher {
    use soup::{QueryBuilderExt, NodeExt};
    use regex::Regex;
    use crate::structures::main::{Novel, Chapter};
    use std::error::Error;
    use sqlx::{
        prelude::*,
        sqlite::SqlitePool,
        Error as SqlError,
    };
    use crate::Db;
    pub async fn handle(db: &SqlitePool, link: String, c_id: String, g_id: String) -> Result<(), String> {
        let before = fetch(link).await;
        let result = match before {
            Ok(x) => x,
            Err(e) => return Err("Invalid link provided".to_string())
        };
        let new_novel_unhandled = handle_soup(result).await;
        let new_novel = match new_novel_unhandled {
            Some(n) => n,
            None => return Err("Unable to locate chapters".to_string())
        };



    }
    pub async fn insert_into_db(db: &SqlitePool, c_id: String, g_id: String, n: Novel) -> Result<u64, SqlError> {
        /// Inserts the Novel into the database to be monitored.
        /// Additionally add the channel_id and guild_id
        let result = sqlx::query("INSERT INTO boxnovel
                VALUES (?, ?, ?, ?)
        ");
    }
    async fn fetch(lk: String) -> Result<String, reqwest::Error>{
        let client = reqwest::Client::new();
        let result = client.post(&lk)
            .send()
            .await?
            .text()
            .await?;
        Ok(result)
    }

    async fn handle_soup(s: String) -> Option<Novel> {
        let soup = soup::Soup::new(&s);
        let result = soup.tag("div")
            .attr("class", "post-title")
            .find();
        let title = match result {
            Some(x) => x.text(),
            None => return None,
        };
        let ch_titles: Vec<String> = soup.tag("li")
            .attr("class", "wp-manga-chapter")
            .limit(30)
            .find_all()
            .filter_map(|r| r.tag("a").find())
            .map(|r| clear(r.text()))
            .collect::<Vec<String>>();
        let ch_links: Vec<String> = soup.tag("div")
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
        let vec_chapters:Vec<Chapter> = ch_titles
            .iter().rev()
            .zip(ch_links.iter().rev())
            .map(|(title, link)| Chapter::new(title.to_string(), link.to_string()))
            .collect::<Vec<Chapter>>();

        Some(Novel::new(title, s, vec_chapters))
    }

    fn clear(s: String) -> String {
        let re = Regex::new(r"[^a-zA-Z0-9]\s").unwrap();

        // REMOVES ALL THE WEIRD WHITESPACES, TABS, NEW LINES
        let s = s.replace("	", "");
        let s = s.replace("\n", "");
        let s = re.replace_all(&s, "").to_string();
        s
    }
}