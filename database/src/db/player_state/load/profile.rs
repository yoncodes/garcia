use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let profile_unlocks = sqlx::query_as::<_, (i32, i32)>(
        "SELECT profile_type, profile_id FROM player_profile_unlocks
         WHERE uid = ? ORDER BY profile_type, profile_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(profile_type, profile_id)| ProfileUnlockRecord {
        profile_type,
        profile_id,
    })
    .collect();
    let skins =
        sqlx::query_scalar("SELECT skin_id FROM player_skins WHERE uid = ? ORDER BY skin_id")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let ship_tags =
        sqlx::query_scalar("SELECT tag_id FROM player_ship_tags WHERE uid = ? ORDER BY tag_id")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let archive_unlocks = sqlx::query_as::<_, (i32, i32)>(
        "SELECT archive_id, unlocked_at FROM player_archive_unlocks
         WHERE uid = ? ORDER BY archive_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(archive_id, unlocked_at)| ArchiveUnlockRecord {
        archive_id,
        unlocked_at,
    })
    .collect();

    player.profile_unlocks = profile_unlocks;
    player.skins = skins;
    player.ship_tags = ship_tags;
    player.archive_unlocks = archive_unlocks;
    Ok(())
}
