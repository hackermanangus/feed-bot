pub mod boxnovel_fetcher {
    use soup::{QueryBuilderExt, NodeExt};
    use regex::Regex;
    use crate::structures::main::{Novel, Chapter};

    async fn fetch(lk: String) {
        let client = reqwest::Client::new();
        let result = client.post(lk)
            .send()
            .await?
            .text()
            .await?;
    }

    async fn handle_soup(s: String) -> Novel {
        let soup = soup::Soup::new(&s);
        let title = soup.tag("div")
            .attr("class", "post-title")
            .find()?
            .text();
        let ch_titles: Vec<String> = soup.tag("li")
            .attr("class", "wp-manga-chapter")
            .limit(30)
            .find_all()
            .filter_map(|r| r.tag("a").find())
            .map(|r| clear(r.text())?)
            .collect();
        let ch_links: Vec<String> = soup.tag("div")
            .attr("class", "wp-manga-chapter")
            .limit(30)
            .find_all()
            .filter_map(|r| r.tag("a").find())
            .map(|r| r.get("href"))
            .collect();
        let mut vec_chapters: Vec<Chapter> = vec![];
        for (i, j) in ch_titles.iter().rev().zip(ch_links.iter().rev()) {
            vec_chapters.push(
                Chapter::new(i.clone().to_string(), j.clone().to_string())
            )
        }
        Novel::new(title, s, vec_chapters)
    }

    async fn clear(s: String) -> String {
        let re = Regex::new(r"[^a-zA-Z0-9]\s").unwrap();

        // REMOVES ALL THE WEIRD WHITESPACES, TABS, NEW LINES
        let s = s.replace("	", "");
        let s = s.replace("\n", "");
        let s = re.replace_all(&s, "").to_string();
        s
    }
}