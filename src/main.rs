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

    let client = Arc::new(build_client(&config).await?);

    let _client = client.clone();
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
        let count_entity = spam_count_map.get_or_insert(sender.to_string(), AtomicUsize::new(0));
        if config.spam_keywords.iter().any(|key| body.contains(key)) {
            count_entity
                .value()
                .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        } else {
            count_entity
                .value()
                .store(0, std::sync::atomic::Ordering::Release);
        }

        if count_entity
            .value()
            .load(std::sync::atomic::Ordering::Acquire)
            >= config.spam_limit as _
        {
            for room in _client.joined_rooms() {
                if let Err(e) = room.ban_user(&sender, Some("Spam")).await {
                    println!(
                        "Sorry, I cannot ban {sender} from {}: {e}",
                        room.name().as_deref().unwrap_or("Unknown")
                    );
                }
            }
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
