use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_profile_unlocks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for profile in &player.profile_unlocks {
        sqlx::query(
            "INSERT INTO player_profile_unlocks (uid, profile_type, profile_id) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(profile.profile_type)
        .bind(profile.profile_id)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}
