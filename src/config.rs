use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub spam_limit: u32,
    pub spam_keywords: Vec<String>,
}
