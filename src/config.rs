use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,

    pub spam_limit: u32,
    pub spam_regex_exprs: Vec<String>,

    pub http_proxy: Option<String>,
}
