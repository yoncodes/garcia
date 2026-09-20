use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let gold_coins = sqlx::query_as::<_, (i32, String)>(
        "SELECT city_id, coin_id FROM player_gold_coins WHERE uid = ? ORDER BY city_id, coin_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(city_id, coin_id)| GoldCoinRecord { city_id, coin_id })
    .collect();
    let collection_resources = sqlx::query_as::<_, (String, i32)>(
        "SELECT collection_id, remaining FROM player_collection_resources
         WHERE uid = ? ORDER BY collection_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(collection_id, remaining)| CollectionResourceRecord {
        collection_id,
        remaining,
    })
    .collect();
    let monster_points = sqlx::query_as::<_, (i32, String)>(
        "SELECT day, point_id FROM player_monster_points WHERE uid = ? ORDER BY point_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(day, point_id)| MonsterPointRecord { day, point_id })
    .collect();
    let monster_manuals = sqlx::query_as::<_, (i32, i64)>(
        "SELECT gameplay_id, enemy_hash FROM player_monster_manuals WHERE uid = ? ORDER BY gameplay_id, rowid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(gameplay_id, enemy_hash)| {
        Ok(MonsterManualRecord {
            gameplay_id,
            enemy_hash: u32::try_from(enemy_hash)
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
        })
    })
    .collect::<sqlx::Result<Vec<_>>>()?;
    let region_coin_daily = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT day, coin_id, amount FROM player_region_coin_daily WHERE uid = ? ORDER BY coin_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(day, coin_id, amount)| RegionCoinDailyRecord {
        day,
        coin_id,
        amount,
    })
    .collect();
    let region_ticket_day =
        sqlx::query_scalar::<_, i32>("SELECT ticket_day FROM player_region_daily WHERE uid = ?")
            .bind(uid)
            .fetch_optional(pool)
            .await?
            .unwrap_or(i32::MIN);
    let region_levels = sqlx::query_as::<_, (i32, i32)>(
        "SELECT region_id, level FROM player_region_progress WHERE uid = ? ORDER BY region_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(region_id, level)| RegionProgressRecord { region_id, level })
    .collect();
    let reward_boxes = sqlx::query_as::<_, (String, i32)>(
        "SELECT id, status FROM player_reward_boxes WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, status)| RewardBoxRecord { id, status })
    .collect();

    player.gold_coins = gold_coins;
    player.collection_resources = collection_resources;
    player.monster_points = monster_points;
    player.monster_manuals = monster_manuals;
    player.region_coin_daily = region_coin_daily;
    player.region_ticket_day = region_ticket_day;
    player.region_levels = region_levels;
    player.reward_boxes = reward_boxes;
    Ok(())
}
