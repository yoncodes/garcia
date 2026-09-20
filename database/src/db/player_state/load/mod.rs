use sqlx::SqlitePool;

use crate::models::game::player_state::PlayerRecord;

mod activities;
mod checkins;
mod collection;
mod communication;
mod equipment;
mod exploration;
mod inventory;
mod profile;
mod progression;
mod world;

pub async fn load(pool: &SqlitePool, uid: i64) -> sqlx::Result<Option<PlayerRecord>> {
    let Some(row) = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        ),
    >(
        "SELECT uid, username, nickname, level, gameplay_id, created_at, last_login_time,
                cur_form, profile_avatar, profile_card, profile_title, profile_frame, region,
                banner_girl, traced_task_group, heat_updated_at
         FROM players WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let mut player = PlayerRecord {
        uid: row.0,
        username: row.1,
        nickname: row.2,
        level: row.3,
        gameplay_id: row.4,
        created_at: row.5,
        last_login_time: row.6,
        cur_form: row.7,
        profile_avatar: row.8,
        profile_card: row.9,
        profile_title: row.10,
        profile_frame: row.11,
        region: row.12,
        banner_girl: row.13,
        traced_task_group: row.14,
        heat_updated_at: row.15,
        ..Default::default()
    };

    communication::load(pool, uid, &mut player).await?;
    inventory::load(pool, uid, &mut player).await?;
    checkins::load(pool, uid, &mut player).await?;
    equipment::load(pool, uid, &mut player).await?;
    collection::load(pool, uid, &mut player).await?;
    world::load(pool, uid, &mut player).await?;
    progression::load(pool, uid, &mut player).await?;
    exploration::load(pool, uid, &mut player).await?;
    activities::load(pool, uid, &mut player).await?;
    profile::load(pool, uid, &mut player).await?;

    Ok(Some(player))
}
