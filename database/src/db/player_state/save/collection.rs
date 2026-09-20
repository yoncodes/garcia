use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_collections WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for collection in &player.collections {
        sqlx::query(
            "INSERT INTO player_collections (uid, cid, in_time, reward) VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(collection.cid)
        .bind(collection.in_time)
        .bind(collection.reward)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_collection_suit_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward in &player.collection_suit_rewards {
        sqlx::query(
            "INSERT INTO player_collection_suit_rewards (uid, suit_id, step) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(reward.suit_id)
        .bind(reward.step)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_collection_places WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for place in &player.collection_places {
        sqlx::query(
            "INSERT INTO player_collection_places (uid, platform_id, collection_id)
             VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(place.platform_id)
        .bind(place.collection_id)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}
