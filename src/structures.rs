#[derive(Debug)]
pub struct Chapter {
    pub title: String,
    pub link: String,
}

impl Chapter {
    pub fn new(title: String, link: String) -> Chapter {
        Chapter {
            title,
            link,
        }
    }
}

#[derive(Debug)]
pub struct Novel {
    pub title: String,
    pub link: String,
    pub chapters: Vec<Chapter>,
}

impl Novel {
    pub fn new(title: String, link: String, chapters: Vec<Chapter>) -> Novel {
        Novel {
            title,
            link,
            chapters,
        }
    }
    pub fn convert(&self) -> String {
        let result: String = self.chapters
            .iter()
            .map(|s| format!("{} ", s.link))
            .collect::<String>();
        result
    }
}

#[derive(Debug)]
pub struct SQLResultBoxnovel {
    pub guild_id: String,
    pub channel_id: String,
    pub novel: String,
    pub current: String
}
impl SQLResultBoxnovel {
    pub async fn convert(&self) -> Vec<String> {
        self.current
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    }
}