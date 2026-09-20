use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.achievements = player
        .achievements
        .iter()
        .map(
            |achievement| database::models::game::player_state::AchievementRecord {
                id: achievement.id,
                took_at: achievement.took_at,
            },
        )
        .collect();
    record.activity_7day_claims = player
        .activity_7day_claims
        .iter()
        .map(
            |claim| database::models::game::player_state::Activity7DayClaimRecord {
                id: claim.id,
                took_at: claim.took_at,
            },
        )
        .collect();
    record.activity_7day_point_claims = player.activity_7day_point_claims.clone();
    record.activity_level_rewards = player.activity_level_rewards.clone();
    record.activity_limited_level_rewards = player.activity_limited_level_rewards.clone();
    record.activity_challenge_point_claims = player.activity_challenge_point_claims.clone();
    record.version_challenges = player
        .version_challenges
        .iter()
        .map(
            |challenge| database::models::game::player_state::VersionChallengeRecord {
                id: challenge.id,
                star1: challenge.star1,
                star2: challenge.star2,
                star3: challenge.star3,
            },
        )
        .collect();
    record.version_challenge_rewards = player
        .version_challenges
        .iter()
        .flat_map(|challenge| challenge.received_rewards.iter().copied())
        .collect();
    record.more_team_challenges = player
        .more_team_challenges
        .iter()
        .map(
            |challenge| database::models::game::player_state::MoreTeamChallengeRecord {
                id: challenge.info.id,
                cost_time: challenge.info.cost_time,
                star1: challenge.info.star1,
                star2: challenge.info.star2,
                star3: challenge.info.star3,
                cycle: challenge.cycle,
            },
        )
        .collect();
    record.more_team_challenge_rewards = player
        .more_team_challenges
        .iter()
        .flat_map(|challenge| challenge.info.received_rewards.iter().copied())
        .collect();
    record.gachas = player
        .gachas
        .iter()
        .map(|gacha| database::models::game::player_state::GachaRecord {
            gacha_id: gacha.gacha_id,
            all_count: gacha.all_count,
            gacha_count_10: gacha.gacha_count_10,
            reward_cnt: gacha.reward_cnt,
            reward_num: gacha.reward_num,
            taken_new_reward: gacha.taken_new_reward,
            had_take_reward: gacha.had_take_reward,
        })
        .collect();
    record.gacha_logs = player
        .gacha_logs
        .iter()
        .map(|log| database::models::game::player_state::GachaLogRecord {
            gacha_id: log.gacha_id,
            reward: log.reward,
            created_at: log.time,
        })
        .collect();
    record.checkins = player
        .checkins
        .iter()
        .map(
            |checkin| database::models::game::player_state::CheckinRecord {
                aid: checkin.aid,
                check_days: checkin.check_days,
                last_check_day: checkin.last_check_day,
                claimed_days: checkin.claimed_days.clone(),
            },
        )
        .collect();
    record.battle_pass = database::models::game::player_state::BattlePassRecord {
        id: player.battle_pass.id,
        paid_status: player.battle_pass.paid_status,
        level: player.battle_pass.level,
        exp: player.battle_pass.exp,
        exp_week: player.battle_pass.exp_week,
        week: player.battle_pass.week,
        rewards: player
            .battle_pass
            .rewards
            .iter()
            .map(|(&level, &state)| (level, state))
            .collect(),
        tasks: player.battle_pass.tasks.clone(),
    };
    record.mall_purchases = player
        .mall_purchases
        .iter()
        .map(
            |purchase| database::models::game::player_state::MallPurchaseRecord {
                goods_id: purchase.goods_id,
                bought: purchase.bought,
                period: purchase.period,
            },
        )
        .collect();
    record.recharge_purchases = player
        .recharge_purchases
        .iter()
        .map(
            |purchase| database::models::game::player_state::RechargePurchaseRecord {
                recharge_id: purchase.recharge_id,
                purchase_count: purchase.purchase_count,
                last_bought_at: purchase.last_bought_at,
                expires_at: purchase.expires_at,
            },
        )
        .collect();
    record.boss_rush = database::models::game::player_state::BossRushRecord {
        id: player.boss_rush.id,
        ports: player
            .boss_rush
            .ports
            .iter()
            .map(
                |port| database::models::game::player_state::BossRushPortRecord {
                    pid: port.pid,
                    role_ids: port.role_ids.clone(),
                    damage: port.damage,
                },
            )
            .collect(),
        rewards: player.boss_rush.rewards.clone(),
        ranking_rewards: player.boss_rush.ranking_rewards.clone(),
    };
    record.activity_boss = database::models::game::player_state::ActivityBossRecord {
        aid: player.activity_boss.aid,
        day: player.activity_boss.day,
        damage: player.activity_boss.damage,
        daily_damage: player.activity_boss.daily_damage,
        role_ids: player.activity_boss.role_ids.clone(),
        rewards: player.activity_boss.rewards.clone(),
        daily_rewards: player
            .activity_boss
            .daily_rewards
            .iter()
            .map(|(&id, &state)| (id, state))
            .collect(),
    };
    record.rift = database::models::game::player_state::RiftRecord {
        id: player.rift.id,
        current_port_id: player.rift.current_port_id,
        best_buff_score: player.rift.best_buff_score,
        best_score: player.rift.best_score,
        best_stage: player.rift.best_stage,
        completion_count: player.rift.completion_count,
        run_buff_score: player.rift.run_buff_score,
        run_score: player.rift.run_score,
        run_duration: player.rift.run_duration,
        active: player.rift.active,
        roles: player.rift.roles.clone(),
        tasks: player.rift.tasks.clone(),
    };
    record.rouge_progress = player
        .rouge_progress
        .iter()
        .map(|(&id, &pass)| database::models::game::player_state::RougeProgressRecord { id, pass })
        .collect();
    record.rouge_technology = player.rouge_technology.clone();
    record.rouge_score = player.rouge_score;
    record.rouge_score_point_at = player.rouge_score_point_at;
    record.rouge_run = player
        .rouge_run
        .as_ref()
        .map(protocol::prost::Message::encode_to_vec);
}
