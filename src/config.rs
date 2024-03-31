use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub username: String,
    pub auth: Auth,

    pub spam_limit: u32,
    pub spam_regex_exprs: Vec<String>,

    pub http_proxy: Option<String>,
}

#[derive(Deserialize, Default, Serialize)]
#[serde(tag = "type")]
pub enum Auth {
    #[serde(rename = "password")]
    Password { password: String },
    #[serde(rename = "sso_login")]
    #[default]
    SSO,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: "@alice:example.org".into(),
            auth: Auth::Password {
                password: "hardpassword".into(),
            },
            spam_limit: 3,
            spam_regex_exprs: vec![],
            http_proxy: None,
        }
    }
}
