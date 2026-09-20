use sqlx::SqlitePool;

use crate::models::game::player_state::PlayerRecord;

mod activities;
mod collection;
mod communication;
mod exploration;
mod inventory;
mod profile;
mod profile_unlocks;
mod progression;
mod team_loadout;
mod world;

pub async fn save(pool: &SqlitePool, player: &PlayerRecord) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO players (
            uid, username, nickname, level, gameplay_id, created_at, last_login_time,
            cur_form, profile_avatar, profile_card, profile_title, profile_frame, region, banner_girl,
            traced_task_group, heat_updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
            username = excluded.username,
            nickname = excluded.nickname,
            level = excluded.level,
            gameplay_id = excluded.gameplay_id,
            last_login_time = excluded.last_login_time,
            cur_form = excluded.cur_form,
            profile_avatar = excluded.profile_avatar,
            profile_card = excluded.profile_card,
            profile_title = excluded.profile_title,
            profile_frame = excluded.profile_frame,
            region = excluded.region,
            banner_girl = excluded.banner_girl,
            traced_task_group = excluded.traced_task_group,
            heat_updated_at = excluded.heat_updated_at",
    )
    .bind(player.uid)
    .bind(&player.username)
    .bind(&player.nickname)
    .bind(player.level)
    .bind(player.gameplay_id)
    .bind(player.created_at)
    .bind(player.last_login_time)
    .bind(player.cur_form)
    .bind(player.profile_avatar)
    .bind(player.profile_card)
    .bind(player.profile_title)
    .bind(player.profile_frame)
    .bind(player.region)
    .bind(player.banner_girl)
    .bind(player.traced_task_group)
    .bind(player.heat_updated_at)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO player_heat_exchange (uid, day, exchange_count) VALUES (?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET day = excluded.day, exchange_count = excluded.exchange_count",
    )
    .bind(player.uid)
    .bind(player.heat_exchange_day)
    .bind(player.heat_exchange_count)
    .execute(&mut *tx)
    .await?;

    profile_unlocks::save(&mut tx, player).await?;
    team_loadout::save(&mut tx, player).await?;
    profile::save(&mut tx, player).await?;
    inventory::save(&mut tx, player).await?;
    collection::save(&mut tx, player).await?;
    world::save(&mut tx, player).await?;
    progression::save(&mut tx, player).await?;
    exploration::save(&mut tx, player).await?;
    activities::save(&mut tx, player).await?;
    communication::save(&mut tx, player).await?;

    tx.commit().await
}
