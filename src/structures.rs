#[derive(Debug)]
pub struct SQLResultBoxnovel {
    pub guild_id: String,
    pub channel_id: String,
    pub novel: String,
    pub current: String,
}

impl SQLResultBoxnovel {
    pub async fn convert(&self) -> Vec<String> {
        self.current
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    }
}