use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let achievements = sqlx::query_as::<_, (i32, i64)>(
        "SELECT id, took_at FROM player_achievements WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, took_at)| AchievementRecord { id, took_at })
    .collect();
    let activity_7day_claims = sqlx::query_as::<_, (i32, i64)>(
        "SELECT id, took_at FROM player_activity_7day_claims WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, took_at)| Activity7DayClaimRecord { id, took_at })
    .collect();
    let activity_7day_point_claims = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM player_activity_7day_point_claims WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let activity_level_rewards = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM player_activity_level_rewards WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let activity_limited_level_rewards = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM player_activity_limited_level_rewards WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let gachas = sqlx::query_as::<_, (i32, i32, i32, i32, i32, bool, bool)>(
        "SELECT gacha_id, all_count, gacha_count_10, reward_cnt, reward_num,
                taken_new_reward, had_take_reward
         FROM player_gachas WHERE uid = ? ORDER BY gacha_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(
            gacha_id,
            all_count,
            gacha_count_10,
            reward_cnt,
            reward_num,
            taken_new_reward,
            had_take_reward,
        )| {
            GachaRecord {
                gacha_id,
                all_count,
                gacha_count_10,
                reward_cnt,
                reward_num,
                taken_new_reward,
                had_take_reward,
            }
        },
    )
    .collect();
    let gacha_logs = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT gacha_id, reward, created_at
         FROM player_gacha_logs WHERE uid = ? ORDER BY log_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(gacha_id, reward, created_at)| GachaLogRecord {
        gacha_id,
        reward,
        created_at,
    })
    .collect();
    let mut battle_pass = sqlx::query_as::<_, (i32, i32, i32, i32, i32, i32)>(
        "SELECT id, paid_status, level, exp, exp_week, week
         FROM player_battle_pass WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .map_or_else(
        BattlePassRecord::default,
        |(id, paid_status, level, exp, exp_week, week)| BattlePassRecord {
            id,
            paid_status,
            level,
            exp,
            exp_week,
            week,
            ..Default::default()
        },
    );
    battle_pass.rewards = sqlx::query_as::<_, (i32, i32)>(
        "SELECT level, state FROM player_battle_pass_rewards WHERE uid = ? ORDER BY level",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    battle_pass.tasks = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM player_battle_pass_tasks WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let mall_purchases = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT goods_id, bought, period FROM player_mall_purchases WHERE uid = ? ORDER BY goods_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(goods_id, bought, period)| MallPurchaseRecord {
        goods_id,
        bought,
        period,
    })
    .collect();
    let recharge_purchases = sqlx::query_as::<_, (i32, i32, i32, i32)>(
        "SELECT recharge_id, purchase_count, last_bought_at, expires_at
         FROM player_recharge_purchases WHERE uid = ? ORDER BY recharge_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(recharge_id, purchase_count, last_bought_at, expires_at)| RechargePurchaseRecord {
            recharge_id,
            purchase_count,
            last_bought_at,
            expires_at,
        },
    )
    .collect();
    let mut boss_rush = BossRushRecord {
        id: sqlx::query_scalar("SELECT bid FROM player_boss_rush WHERE uid = ?")
            .bind(uid)
            .fetch_optional(pool)
            .await?
            .unwrap_or_default(),
        ..Default::default()
    };
    let boss_port_rows = sqlx::query_as::<_, (i32, i64)>(
        "SELECT pid, damage FROM player_boss_rush_ports WHERE uid = ? ORDER BY pid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    for (pid, damage) in boss_port_rows {
        let role_ids = sqlx::query_scalar(
            "SELECT role_id FROM player_boss_rush_port_roles
             WHERE uid = ? AND pid = ? ORDER BY position",
        )
        .bind(uid)
        .bind(pid)
        .fetch_all(pool)
        .await?;
        boss_rush.ports.push(BossRushPortRecord {
            pid,
            role_ids,
            damage,
        });
    }
    boss_rush.rewards =
        sqlx::query_scalar("SELECT sid FROM player_boss_rush_rewards WHERE uid = ? ORDER BY sid")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    boss_rush.ranking_rewards = sqlx::query_scalar(
        "SELECT bid FROM player_boss_rush_ranking_rewards WHERE uid = ? ORDER BY bid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let mut activity_boss = sqlx::query_as::<_, (i32, i32, i64, i64)>(
        "SELECT aid, day, damage, daily_damage FROM player_activity_boss WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .map_or_else(ActivityBossRecord::default, |row| ActivityBossRecord {
        aid: row.0,
        day: row.1,
        damage: row.2,
        daily_damage: row.3,
        ..Default::default()
    });
    activity_boss.role_ids = sqlx::query_scalar(
        "SELECT role_id FROM player_activity_boss_roles WHERE uid = ? ORDER BY position",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    activity_boss.rewards = sqlx::query_scalar(
        "SELECT reward_id FROM player_activity_boss_rewards
         WHERE uid = ? AND reward_type = 1 ORDER BY reward_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    activity_boss.daily_rewards = sqlx::query_as(
        "SELECT reward_id, state FROM player_activity_boss_rewards
         WHERE uid = ? AND reward_type = 2 ORDER BY reward_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let version_challenges = sqlx::query_as::<_, (i32, bool, bool, bool)>(
        "SELECT challenge_id, star1, star2, star3
         FROM player_version_challenges WHERE uid = ? ORDER BY challenge_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, star1, star2, star3)| VersionChallengeRecord {
        id,
        star1,
        star2,
        star3,
    })
    .collect();
    let version_challenge_rewards = sqlx::query_scalar(
        "SELECT reward_id FROM player_version_challenge_rewards WHERE uid = ? ORDER BY reward_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let more_team_challenges = sqlx::query_as::<_, (i32, i32, bool, bool, bool, i32)>(
        "SELECT challenge_id, cost_time, star1, star2, star3, cycle
         FROM player_more_team_challenges WHERE uid = ? ORDER BY challenge_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(id, cost_time, star1, star2, star3, cycle)| MoreTeamChallengeRecord {
            id,
            cost_time,
            star1,
            star2,
            star3,
            cycle,
        },
    )
    .collect();
    let more_team_challenge_rewards = sqlx::query_scalar(
        "SELECT reward_id FROM player_more_team_challenge_rewards WHERE uid = ? ORDER BY reward_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let mut rift = sqlx::query_as::<_, (i32, i32, i32, i32, i32, i32, i32, i32, i32, bool)>(
        "SELECT rid, current_port_id, best_buff_score, best_score, best_stage, completion_count,
                run_buff_score, run_score, run_duration, active FROM player_rift WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .map_or_else(RiftRecord::default, |row| RiftRecord {
        id: row.0,
        current_port_id: row.1,
        best_buff_score: row.2,
        best_score: row.3,
        best_stage: row.4,
        completion_count: row.5,
        run_buff_score: row.6,
        run_score: row.7,
        run_duration: row.8,
        active: row.9,
        roles: Vec::new(),
        tasks: Vec::new(),
    });
    rift.roles =
        sqlx::query_scalar("SELECT role_id FROM player_rift_roles WHERE uid = ? ORDER BY position")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    rift.tasks =
        sqlx::query_scalar("SELECT task_id FROM player_rift_tasks WHERE uid = ? ORDER BY task_id")
            .bind(uid)
            .fetch_all(pool)
            .await?;

    let rouge_progress = sqlx::query_as::<_, (i32, i32)>(
        "SELECT rouge_id, pass_count FROM player_rouge_progress WHERE uid = ? ORDER BY rouge_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, pass)| RougeProgressRecord { id, pass })
    .collect();
    let rouge_technology = sqlx::query_scalar(
        "SELECT technology_id FROM player_rouge_technology WHERE uid = ? ORDER BY technology_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let (rouge_score, rouge_score_point_at) =
        sqlx::query_as("SELECT score, score_point_at FROM player_rouge_weekly WHERE uid = ?")
            .bind(uid)
            .fetch_optional(pool)
            .await?
            .unwrap_or_default();
    let rouge_run = sqlx::query_scalar("SELECT data FROM player_rouge_run WHERE uid = ?")
        .bind(uid)
        .fetch_optional(pool)
        .await?;

    player.achievements = achievements;
    player.activity_7day_claims = activity_7day_claims;
    player.activity_7day_point_claims = activity_7day_point_claims;
    player.activity_level_rewards = activity_level_rewards;
    player.activity_limited_level_rewards = activity_limited_level_rewards;
    player.gachas = gachas;
    player.gacha_logs = gacha_logs;
    player.battle_pass = battle_pass;
    player.mall_purchases = mall_purchases;
    player.recharge_purchases = recharge_purchases;
    player.boss_rush = boss_rush;
    player.activity_boss = activity_boss;
    player.version_challenges = version_challenges;
    player.version_challenge_rewards = version_challenge_rewards;
    player.more_team_challenges = more_team_challenges;
    player.more_team_challenge_rewards = more_team_challenge_rewards;
    player.rift = rift;
    player.rouge_progress = rouge_progress;
    player.rouge_technology = rouge_technology;
    player.rouge_score = rouge_score;
    player.rouge_score_point_at = rouge_score_point_at;
    player.rouge_run = rouge_run;
    Ok(())
}
