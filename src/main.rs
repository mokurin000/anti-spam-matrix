mod config;
use config::Config;
mod auth;
use auth::{password_login, sso_login};
mod utils;
use utils::ban_user_in_room;

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
use regex::RegexSet;
use std::{
    fs,
    path::PathBuf,
    process,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let project_dir = directories::ProjectDirs::from("io", "poly000", PACKAGE_NAME)
        .map(|d| d.config_dir().to_owned())
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&project_dir)?;
    let config_path = project_dir.join("config.toml");
    let auth_path = project_dir.join("auth.json");

    if fs::File::open(&config_path).is_err() {
        println!("'config.toml' not exists, generating template...");
        fs::write(&config_path, toml::to_string_pretty(&Config::default())?)?;
        println!(
            "successfully generated at {}.",
            config_path.as_os_str().to_string_lossy()
        );
        process::exit(1);
    }
    let config: Arc<Config> = Arc::new(toml::from_str(&fs::read_to_string(config_path)?)?);
    let spam_count_map: Arc<SkipMap<String, AtomicUsize>> = Arc::new(SkipMap::new());

    let client = Arc::new(build_client(&config, &auth_path).await?);

    let _client = client.clone();
    let regex_set = RegexSet::new(&config.spam_regex_exprs)?;
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
        if regex_set.is_match(body) {
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
                ban_user_in_room(&room, &sender).await;
            }
        }
    });

    client.sync(SyncSettings::default()).await?;

    Ok(())
}

async fn build_client(config: &Config, auth_path: &PathBuf) -> Result<Client> {
    let userid: &UserId = config.username.as_str().try_into()?;
    let mut client = matrix_sdk::Client::builder().server_name(userid.server_name());
    if let Some(proxy) = &config.proxy {
        client = client.proxy(proxy);
    }
    let client = client.build().await?;
    match &config.auth {
        config::Auth::Password { password } => {
            password_login(&client, &userid, &password).await?;
        }
        config::Auth::SSO => {
            sso_login(&client, &auth_path).await?;
        }
    }

    Ok(client)
}

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
