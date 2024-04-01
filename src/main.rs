mod config;
use config::Config;

use crossbeam_skiplist::SkipMap;
use matrix_sdk::{
    config::SyncSettings,
    matrix_auth::{MatrixSession, MatrixSessionTokens},
    ruma::{
        events::{
            room::message::SyncRoomMessageEvent, OriginalSyncMessageLikeEvent, SyncMessageLikeEvent,
        },
        UserId,
    },
    Client, SessionMeta,
};
use regex::RegexSet;
use std::{
    fs, process,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = directories::BaseDirs::new()
        .map(|b| b.config_dir().to_owned().join("config.toml"))
        .unwrap_or("config.toml".into());
    if fs::File::open(&config_path).is_err() {
        println!("'config.toml' not exists, generating template...");
        fs::write("config.toml", toml::to_string_pretty(&Config::default())?)?;
        println!(
            "successfully generated at {}.",
            config_path.as_os_str().to_string_lossy()
        );
        process::exit(1);
    }
    let config: Arc<Config> = Arc::new(toml::from_str(&fs::read_to_string(config_path)?)?);
    let spam_count_map: Arc<SkipMap<String, AtomicUsize>> = Arc::new(SkipMap::new());

    let client = Arc::new(build_client(&config).await?);

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
    match &config.auth {
        config::Auth::Password { password } => {
            client
                .matrix_auth()
                .login_username(userid, password)
                .initial_device_display_name(PACKAGE_NAME)
                .await?;
        }
        config::Auth::SSO => {
            sso_login(&client).await?;
        }
    }

    Ok(client)
}

async fn sso_login(client: &Client) -> Result<()> {
    let auth_path = directories::BaseDirs::new()
        .map(|b| b.config_dir().to_owned().join("auth.json"))
        .unwrap_or_else(|| "auth.json".into());
    if let Ok(auth) = fs::read_to_string(&auth_path) {
        let session: MatrixSession = serde_json::from_str(&auth)?;
        if client.restore_session(session).await.is_ok() {
            return Ok(());
        }
    }

    let auth_resp = client
        .matrix_auth()
        .login_sso(|sso| async move {
            println!("trying to open {sso} from default browser...");
            println!("If this doesn't work, please access the URL above manually.");
            open::that(&sso)?;
            Ok(())
        })
        .initial_device_display_name(PACKAGE_NAME)
        .await?;

    let session = MatrixSession {
        meta: SessionMeta {
            user_id: auth_resp.user_id,
            device_id: auth_resp.device_id,
        },
        tokens: MatrixSessionTokens {
            access_token: auth_resp.access_token,
            refresh_token: auth_resp.refresh_token,
        },
    };
    fs::write(&auth_path, serde_json::to_string_pretty(&session)?)?;
    Ok(())
}

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
