use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let collections = sqlx::query_as::<_, (i32, i64, bool)>(
        "SELECT cid, in_time, reward FROM player_collections WHERE uid = ? ORDER BY cid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(cid, in_time, reward)| CollectionRecord {
        cid,
        in_time,
        reward,
    })
    .collect();
    let collection_suit_rewards = sqlx::query_as::<_, (i32, i32)>(
        "SELECT suit_id, step FROM player_collection_suit_rewards
         WHERE uid = ? ORDER BY suit_id, step",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(suit_id, step)| CollectionSuitRewardRecord { suit_id, step })
    .collect();
    let collection_places = sqlx::query_as::<_, (i32, i32)>(
        "SELECT collection_id, platform_id FROM player_collection_places
         WHERE uid = ? ORDER BY platform_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(collection_id, platform_id)| CollectionPlaceRecord {
        collection_id,
        platform_id,
    })
    .collect();

    player.collections = collections;
    player.collection_suit_rewards = collection_suit_rewards;
    player.collection_places = collection_places;
    Ok(())
}
