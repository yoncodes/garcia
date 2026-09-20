use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let role_awards = sqlx::query_as::<_, (i32, i32)>(
        "SELECT role_id, rank FROM player_role_rank_awards WHERE uid = ? ORDER BY role_id, rank",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let role_talents = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT role_id, position, level FROM player_role_talents
         WHERE uid = ? ORDER BY role_id, position",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let role_progress =
        sqlx::query_as::<_, (i32, i32, i32, i64, i32, i32, i32, i32, i32, i32, bool)>(
            "SELECT role_id, level, exp, user_partner_id, position, maid_qua, skin_id,
                appear_skill_key, element, element4call, cur_mc
         FROM player_role_progress WHERE uid = ? ORDER BY role_id",
        )
        .bind(uid)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(
            |(
                role_id,
                level,
                exp,
                user_partner_id,
                position,
                maid_qua,
                skin_id,
                appear_skill_key,
                element,
                element4call,
                cur_mc,
            )| RoleProgressRecord {
                role_id,
                level,
                exp,
                user_partner_id,
                position,
                maid_qua,
                skin_id,
                appear_skill_key,
                element,
                element4call,
                cur_mc,
                awards: role_awards
                    .iter()
                    .filter_map(|&(award_role, rank)| (award_role == role_id).then_some(rank))
                    .collect(),
                talents: role_talents
                    .iter()
                    .filter_map(|&(talent_role, position, level)| {
                        (talent_role == role_id).then_some((position, level))
                    })
                    .collect(),
            },
        )
        .collect();
    let (daily_day, daily_activity, daily_reward_progress) = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT day, activity, reward_progress FROM player_daily_state WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .unwrap_or_default();
    let (heat_exchange_day, heat_exchange_count) = sqlx::query_as::<_, (i32, i32)>(
        "SELECT day, exchange_count FROM player_heat_exchange WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .unwrap_or_default();
    let daily_tasks = sqlx::query_as::<_, (i32, bool, i32, i32)>(
        "SELECT id, taken, progress, total FROM player_daily_tasks WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, taken, progress, total)| DailyTaskRecord {
        id,
        taken,
        progress,
        total,
    })
    .collect();

    player.role_progress = role_progress;
    player.daily_day = daily_day;
    player.daily_activity = daily_activity;
    player.daily_reward_progress = daily_reward_progress;
    player.heat_exchange_day = heat_exchange_day;
    player.heat_exchange_count = heat_exchange_count;
    player.daily_tasks = daily_tasks;
    Ok(())
}
