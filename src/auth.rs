use crate::PACKAGE_NAME;
use std::{fs, path::PathBuf};

use anyhow::Result;
use matrix_sdk::{
    authentication::matrix::MatrixSession, ruma::UserId, Client, SessionMeta, SessionTokens,
};

pub async fn password_login(
    client: &Client,
    userid: &UserId,
    password: impl AsRef<str>,
) -> Result<()> {
    client
        .matrix_auth()
        .login_username(userid, password.as_ref())
        .initial_device_display_name(PACKAGE_NAME)
        .await?;

    Ok(())
}

pub async fn sso_login(client: &Client, auth_path: &PathBuf) -> Result<()> {
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
        tokens: SessionTokens {
            access_token: auth_resp.access_token,
            refresh_token: auth_resp.refresh_token,
        },
    };
    fs::write(&auth_path, serde_json::to_string_pretty(&session)?)?;
    Ok(())
}
