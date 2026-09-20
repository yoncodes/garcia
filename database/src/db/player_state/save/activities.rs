use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_achievements WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for achievement in &player.achievements {
        sqlx::query("INSERT INTO player_achievements (uid, id, took_at) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(achievement.id)
            .bind(achievement.took_at)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_activity_7day_claims WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for claim in &player.activity_7day_claims {
        sqlx::query("INSERT INTO player_activity_7day_claims (uid, id, took_at) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(claim.id)
            .bind(claim.took_at)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_activity_7day_point_claims WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.activity_7day_point_claims {
        sqlx::query("INSERT INTO player_activity_7day_point_claims (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_activity_level_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.activity_level_rewards {
        sqlx::query("INSERT INTO player_activity_level_rewards (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_activity_limited_level_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.activity_limited_level_rewards {
        sqlx::query("INSERT INTO player_activity_limited_level_rewards (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_gachas WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for gacha in &player.gachas {
        sqlx::query(
            "INSERT INTO player_gachas (
                uid, gacha_id, all_count, gacha_count_10, reward_cnt, reward_num,
                taken_new_reward, had_take_reward
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(gacha.gacha_id)
        .bind(gacha.all_count)
        .bind(gacha.gacha_count_10)
        .bind(gacha.reward_cnt)
        .bind(gacha.reward_num)
        .bind(gacha.taken_new_reward)
        .bind(gacha.had_take_reward)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_gacha_logs WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for (log_id, log) in player.gacha_logs.iter().enumerate() {
        sqlx::query(
            "INSERT INTO player_gacha_logs (uid, log_id, gacha_id, reward, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(log_id as i64)
        .bind(log.gacha_id)
        .bind(log.reward)
        .bind(log.created_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_checkin_claims WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM player_checkins WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for checkin in &player.checkins {
        sqlx::query(
            "INSERT INTO player_checkins (uid, aid, check_days, last_check_day)
             VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(checkin.aid)
        .bind(checkin.check_days)
        .bind(checkin.last_check_day)
        .execute(&mut *tx)
        .await?;
        for day in &checkin.claimed_days {
            sqlx::query("INSERT INTO player_checkin_claims (uid, aid, get_day) VALUES (?, ?, ?)")
                .bind(player.uid)
                .bind(checkin.aid)
                .bind(day)
                .execute(&mut *tx)
                .await?;
        }
    }
    sqlx::query(
        "INSERT INTO player_battle_pass (uid, id, paid_status, level, exp, exp_week, week)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET id = excluded.id, paid_status = excluded.paid_status,
            level = excluded.level, exp = excluded.exp, exp_week = excluded.exp_week,
            week = excluded.week",
    )
    .bind(player.uid)
    .bind(player.battle_pass.id)
    .bind(player.battle_pass.paid_status)
    .bind(player.battle_pass.level)
    .bind(player.battle_pass.exp)
    .bind(player.battle_pass.exp_week)
    .bind(player.battle_pass.week)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_battle_pass_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for (level, state) in &player.battle_pass.rewards {
        sqlx::query("INSERT INTO player_battle_pass_rewards (uid, level, state) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(level)
            .bind(state)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_battle_pass_tasks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.battle_pass.tasks {
        sqlx::query("INSERT INTO player_battle_pass_tasks (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_mall_purchases WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for purchase in &player.mall_purchases {
        sqlx::query(
            "INSERT INTO player_mall_purchases (uid, goods_id, bought, period) VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(purchase.goods_id)
        .bind(purchase.bought)
        .bind(purchase.period)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO player_boss_rush (uid, bid) VALUES (?, ?)
         ON CONFLICT(uid) DO UPDATE SET bid = excluded.bid",
    )
    .bind(player.uid)
    .bind(player.boss_rush.id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_boss_rush_port_roles WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM player_boss_rush_ports WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for port in &player.boss_rush.ports {
        sqlx::query("INSERT INTO player_boss_rush_ports (uid, pid, damage) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(port.pid)
            .bind(port.damage)
            .execute(&mut *tx)
            .await?;
        for (position, role_id) in port.role_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_boss_rush_port_roles (uid, pid, position, role_id)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(port.pid)
            .bind(position as i32)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query("DELETE FROM player_boss_rush_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for sid in &player.boss_rush.rewards {
        sqlx::query("INSERT INTO player_boss_rush_rewards (uid, sid) VALUES (?, ?)")
            .bind(player.uid)
            .bind(sid)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_boss_rush_ranking_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for bid in &player.boss_rush.ranking_rewards {
        sqlx::query("INSERT INTO player_boss_rush_ranking_rewards (uid, bid) VALUES (?, ?)")
            .bind(player.uid)
            .bind(bid)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_recharge_purchases WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for purchase in &player.recharge_purchases {
        sqlx::query(
            "INSERT INTO player_recharge_purchases
             (uid, recharge_id, purchase_count, last_bought_at, expires_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(purchase.recharge_id)
        .bind(purchase.purchase_count)
        .bind(purchase.last_bought_at)
        .bind(purchase.expires_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO player_activity_boss (uid, aid, day, damage, daily_damage)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
             aid = excluded.aid,
             day = excluded.day,
             damage = excluded.damage,
             daily_damage = excluded.daily_damage",
    )
    .bind(player.uid)
    .bind(player.activity_boss.aid)
    .bind(player.activity_boss.day)
    .bind(player.activity_boss.damage)
    .bind(player.activity_boss.daily_damage)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_activity_boss_roles WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for (position, role_id) in player.activity_boss.role_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO player_activity_boss_roles (uid, position, role_id) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(position as i32)
        .bind(role_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_activity_boss_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward_id in &player.activity_boss.rewards {
        sqlx::query(
            "INSERT INTO player_activity_boss_rewards (uid, reward_type, reward_id, state)
             VALUES (?, 1, ?, 1)",
        )
        .bind(player.uid)
        .bind(reward_id)
        .execute(&mut *tx)
        .await?;
    }
    for (reward_id, state) in &player.activity_boss.daily_rewards {
        sqlx::query(
            "INSERT INTO player_activity_boss_rewards (uid, reward_type, reward_id, state)
             VALUES (?, 2, ?, ?)",
        )
        .bind(player.uid)
        .bind(reward_id)
        .bind(state)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_version_challenges WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for challenge in &player.version_challenges {
        sqlx::query(
            "INSERT INTO player_version_challenges (uid, challenge_id, star1, star2, star3)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(challenge.id)
        .bind(challenge.star1)
        .bind(challenge.star2)
        .bind(challenge.star3)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_version_challenge_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward_id in &player.version_challenge_rewards {
        sqlx::query("INSERT INTO player_version_challenge_rewards (uid, reward_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(reward_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_more_team_challenges WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for challenge in &player.more_team_challenges {
        sqlx::query(
            "INSERT INTO player_more_team_challenges
                (uid, challenge_id, cost_time, star1, star2, star3, cycle)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(challenge.id)
        .bind(challenge.cost_time)
        .bind(challenge.star1)
        .bind(challenge.star2)
        .bind(challenge.star3)
        .bind(challenge.cycle)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_more_team_challenge_rewards WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward_id in &player.more_team_challenge_rewards {
        sqlx::query(
            "INSERT INTO player_more_team_challenge_rewards (uid, reward_id) VALUES (?, ?)",
        )
        .bind(player.uid)
        .bind(reward_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO player_rift (
             uid, rid, current_port_id, best_buff_score, best_score, best_stage, completion_count,
             run_buff_score, run_score, run_duration, active
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
             rid = excluded.rid,
             current_port_id = excluded.current_port_id,
             best_buff_score = excluded.best_buff_score,
             best_score = excluded.best_score,
             best_stage = excluded.best_stage,
             completion_count = excluded.completion_count,
             run_buff_score = excluded.run_buff_score,
             run_score = excluded.run_score,
             run_duration = excluded.run_duration,
             active = excluded.active",
    )
    .bind(player.uid)
    .bind(player.rift.id)
    .bind(player.rift.current_port_id)
    .bind(player.rift.best_buff_score)
    .bind(player.rift.best_score)
    .bind(player.rift.best_stage)
    .bind(player.rift.completion_count)
    .bind(player.rift.run_buff_score)
    .bind(player.rift.run_score)
    .bind(player.rift.run_duration)
    .bind(player.rift.active)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_rift_roles WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for (position, role_id) in player.rift.roles.iter().enumerate() {
        sqlx::query("INSERT INTO player_rift_roles (uid, position, role_id) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(position as i32)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_rift_tasks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for task_id in &player.rift.tasks {
        sqlx::query("INSERT INTO player_rift_tasks (uid, task_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_rouge_progress WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for progress in &player.rouge_progress {
        sqlx::query(
            "INSERT INTO player_rouge_progress (uid, rouge_id, pass_count) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(progress.id)
        .bind(progress.pass)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_rouge_technology WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for technology_id in &player.rouge_technology {
        sqlx::query("INSERT INTO player_rouge_technology (uid, technology_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(technology_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO player_rouge_weekly (uid, score, score_point_at) VALUES (?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
             score = excluded.score,
             score_point_at = excluded.score_point_at",
    )
    .bind(player.uid)
    .bind(player.rouge_score)
    .bind(player.rouge_score_point_at)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM player_rouge_run WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    if let Some(data) = &player.rouge_run {
        sqlx::query("INSERT INTO player_rouge_run (uid, data) VALUES (?, ?)")
            .bind(player.uid)
            .bind(data)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}
