mod config;
use config::Config;

use crossbeam_skiplist::SkipMap;
use matrix_sdk::{
    config::SyncSettings,
    ruma::{
        events::{
            room::message::SyncRoomMessageEvent, OriginalSyncMessageLikeEvent, SyncMessageLikeEvent,
        },
        UserId,
    },
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
    let spam_count_map: Arc<SkipMap<String, AtomicUsize>> = Arc::new(SkipMap::new());

    let client = build_client(&config).await?;

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        let SyncMessageLikeEvent::Original(ev) = ev else {
            return;
        };

        let OriginalSyncMessageLikeEvent {
            content, sender, ..
        } = ev;

        if content.msgtype() != "m.text" {
            return;
        }

        let body = content.msgtype.body();
        if config.spam_keywords.iter().any(|key| body.contains(key)) {
            spam_count_map
                .get_or_insert(sender.to_string(), AtomicUsize::new(1))
                .value()
                .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        } else {
            spam_count_map.insert(sender.to_string(), AtomicUsize::new(1));
        }

        if spam_count_map
            .get_or_insert(sender.to_string(), AtomicUsize::new(1))
            .value()
            .load(std::sync::atomic::Ordering::Acquire)
            > config.spam_limit as _
        {
            println!("ban {sender}");
        }
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
