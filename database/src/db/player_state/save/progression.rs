use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_role_progress WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for role in &player.role_progress {
        sqlx::query(
            "INSERT INTO player_role_progress
                (uid, role_id, level, exp, user_partner_id, position, maid_qua, skin_id,
                 appear_skill_key, element, element4call, cur_mc)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(role.role_id)
        .bind(role.level)
        .bind(role.exp)
        .bind(role.user_partner_id)
        .bind(role.position)
        .bind(role.maid_qua)
        .bind(role.skin_id)
        .bind(role.appear_skill_key)
        .bind(role.element)
        .bind(role.element4call)
        .bind(role.cur_mc)
        .execute(&mut *tx)
        .await?;
        for rank in &role.awards {
            sqlx::query(
                "INSERT INTO player_role_rank_awards (uid, role_id, rank) VALUES (?, ?, ?)",
            )
            .bind(player.uid)
            .bind(role.role_id)
            .bind(rank)
            .execute(&mut *tx)
            .await?;
        }
        for &(position, level) in &role.talents {
            sqlx::query(
                "INSERT INTO player_role_talents (uid, role_id, position, level)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(role.role_id)
            .bind(position)
            .bind(level)
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query(
        "INSERT INTO player_daily_state (uid, day, activity, reward_progress)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
            day = excluded.day,
            activity = excluded.activity,
            reward_progress = excluded.reward_progress",
    )
    .bind(player.uid)
    .bind(player.daily_day)
    .bind(player.daily_activity)
    .bind(player.daily_reward_progress)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_daily_tasks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for task in &player.daily_tasks {
        sqlx::query(
            "INSERT INTO player_daily_tasks (uid, id, taken, progress, total)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(task.id)
        .bind(task.taken)
        .bind(task.progress)
        .bind(task.total)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}
