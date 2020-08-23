#[derive(Debug, Clone)]
pub struct SQLResultBoxnovel {
    pub guild_id: String,
    pub channel_id: String,
    pub title: String,
    pub novel: String,
    pub current: String,
}

impl SQLResultBoxnovel {
    pub async fn convert(&self) -> SQLProcessBoxnovel {
        let  result = self.current
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        SQLProcessBoxnovel {
            g_id: self.guild_id.to_string(),
            c_id: self.channel_id.to_string(),
            title: self.title.to_string(),
            novel: self.novel.to_string(),
            current: result
        }
    }
}
#[derive(Debug, Clone)]
pub struct SQLProcessBoxnovel {
    pub g_id: String,
    pub c_id: String,
    pub title: String,
    pub novel: String,
    pub current: Vec<String>
}
impl SQLProcessBoxnovel {
    pub async fn convert(&self) -> SQLResultBoxnovel {
        let result = self.current.join(" ");
        SQLResultBoxnovel {
            guild_id: self.g_id.to_string(),
            channel_id: self.c_id.to_string(),
            title: self.title.to_string(),
            novel: self.novel.to_string(),
            current: result
        }
    }
}
