use matrix_sdk::{ruma::UserId, Room};

pub async fn ban_user_in_room(room: &Room, sender: &UserId) {
    if let Err(e) = room.ban_user(&sender, Some("Spam")).await {
        println!(
            "Sorry, I cannot ban {sender} from {}: {e}",
            room.name()
                .as_deref()
                .map(str::to_string)
                .or_else(|| room.alt_aliases().first().map(|a| a.alias().to_string()))
                .unwrap_or("Unknown".into())
        );
    };
}
