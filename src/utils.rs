use std::{fs, path::PathBuf, process};

use anyhow::Result;
use matrix_sdk::{ruma::UserId, Room};
use tracing::warn;

use crate::config::Config;
use crate::PACKAGE_NAME;

pub async fn ban_user_in_room(room: &Room, sender: &UserId) {
    if let Err(e) = room.ban_user(&sender, Some("Spam")).await {
        let room_name = room
            .name()
            .as_deref()
            .map(str::to_string)
            .or_else(|| room.alt_aliases().first().map(|a| a.alias().to_string()))
            .unwrap_or("Unknown".into());
        warn!("failed to ban {sender} from {room_name}: {e}");
    };
}

pub fn init_dirs() -> Result<(PathBuf, PathBuf)> {
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

    Ok((config_path, auth_path))
}
