use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_gold_coins WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for coin in &player.gold_coins {
        sqlx::query("INSERT INTO player_gold_coins (uid, city_id, coin_id) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(coin.city_id)
            .bind(&coin.coin_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_collection_resources WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for resource in &player.collection_resources {
        sqlx::query(
            "INSERT INTO player_collection_resources (uid, collection_id, remaining)
             VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(&resource.collection_id)
        .bind(resource.remaining)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_monster_points WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for point in &player.monster_points {
        sqlx::query("INSERT INTO player_monster_points (uid, day, point_id) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(point.day)
            .bind(&point.point_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_monster_manuals WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for monster in &player.monster_manuals {
        sqlx::query(
            "INSERT INTO player_monster_manuals (uid, gameplay_id, enemy_hash) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(monster.gameplay_id)
        .bind(i64::from(monster.enemy_hash))
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_region_coin_daily WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for coin in &player.region_coin_daily {
        sqlx::query(
            "INSERT INTO player_region_coin_daily (uid, day, coin_id, amount) VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(coin.day)
        .bind(coin.coin_id)
        .bind(coin.amount)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO player_region_daily (uid, ticket_day) VALUES (?, ?)
         ON CONFLICT(uid) DO UPDATE SET ticket_day = excluded.ticket_day",
    )
    .bind(player.uid)
    .bind(player.region_ticket_day)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_region_progress WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for region in &player.region_levels {
        sqlx::query("INSERT INTO player_region_progress (uid, region_id, level) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(region.region_id)
            .bind(region.level)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_reward_boxes WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward_box in &player.reward_boxes {
        sqlx::query("INSERT INTO player_reward_boxes (uid, id, status) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(&reward_box.id)
            .bind(reward_box.status)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}
