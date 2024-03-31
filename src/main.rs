mod config;
use config::Config;

use crossbeam_skiplist::SkipMap;
use matrix_sdk::{
    config::SyncSettings,
    ruma::{events::room::message::SyncRoomMessageEvent, UserId},
    Client,
};
use std::{
    fs,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Arc<Config> = Arc::new(toml::from_str(&fs::read_to_string("config.toml")?)?);
    let spam_count_map: SkipMap<String, AtomicUsize> = SkipMap::new();

    let client = build_client(&config).await?;

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        println!("Received a message {ev:?}");
    });

    client.sync(SyncSettings::default()).await?;

    Ok(())
}

async fn build_client(config: &Config) -> Result<Client> {
    let userid: &UserId = config.username.as_str().try_into()?;
    let mut client = matrix_sdk::Client::builder().server_name(userid.server_name());
    if let Some(proxy) = &config.http_proxy {
        client = client.proxy(proxy);
    }
    let client = client.build().await?;
    client
        .matrix_auth()
        .login_username(userid, &config.password)
        .await?;

    Ok(client)
}
