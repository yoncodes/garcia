use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
    tables: &GameTables,
) {
    player.achievements = std::mem::take(&mut record.achievements)
        .into_iter()
        .map(|achievement| DcNetDataAchi {
            id: achievement.id,
            took_at: achievement.took_at,
            ..Default::default()
        })
        .collect();
    player.activity_7day_claims = std::mem::take(&mut record.activity_7day_claims)
        .into_iter()
        .map(|claim| DcNetDataAct7Day {
            id: claim.id,
            took_at: claim.took_at,
            ..Default::default()
        })
        .collect();
    player.activity_7day_point_claims = std::mem::take(&mut record.activity_7day_point_claims);
    player.activity_level_rewards = std::mem::take(&mut record.activity_level_rewards);
    player.activity_limited_level_rewards =
        std::mem::take(&mut record.activity_limited_level_rewards);
    player.activity_challenge_point_claims =
        std::mem::take(&mut record.activity_challenge_point_claims);
    player.version_challenges = std::mem::take(&mut record.version_challenges)
        .into_iter()
        .map(|challenge| DcNetDataVersionChallengeInfo {
            id: challenge.id,
            star1: challenge.star1,
            star2: challenge.star2,
            star3: challenge.star3,
            received_rewards: record
                .version_challenge_rewards
                .iter()
                .filter(|reward_id| {
                    tables
                        .version_challenge_rewards
                        .iter()
                        .any(|reward| reward.id == **reward_id && reward.dungeon_id == challenge.id)
                })
                .copied()
                .collect(),
        })
        .collect();
    player.more_team_challenges = std::mem::take(&mut record.more_team_challenges)
        .into_iter()
        .map(|challenge| MoreTeamChallengeState {
            info: protocol::pbcommon::DcNetDataMoreChallengeInfo {
                id: challenge.id,
                cost_time: challenge.cost_time,
                star1: challenge.star1,
                star2: challenge.star2,
                star3: challenge.star3,
                received_rewards: record
                    .more_team_challenge_rewards
                    .iter()
                    .filter(|reward_id| {
                        tables.more_team_challenge_stars.iter().any(|reward| {
                            reward.id == **reward_id && reward.dungeon_id == challenge.id
                        })
                    })
                    .copied()
                    .collect(),
            },
            cycle: challenge.cycle,
        })
        .collect();
    player.gachas = std::mem::take(&mut record.gachas)
        .into_iter()
        .map(|gacha| GachaState {
            gacha_id: gacha.gacha_id,
            all_count: gacha.all_count,
            gacha_count_10: gacha.gacha_count_10,
            reward_cnt: gacha.reward_cnt,
            reward_num: gacha.reward_num,
            taken_new_reward: gacha.taken_new_reward,
            had_take_reward: gacha.had_take_reward,
        })
        .collect();
    player.gacha_logs = std::mem::take(&mut record.gacha_logs)
        .into_iter()
        .map(|log| DcNetDataGachaLog {
            time: log.created_at,
            reward: log.reward,
            gacha_id: log.gacha_id,
        })
        .collect();
    player.checkins = std::mem::take(&mut record.checkins)
        .into_iter()
        .map(|checkin| CheckinState {
            aid: checkin.aid,
            check_days: checkin.check_days,
            last_check_day: checkin.last_check_day,
            claimed_days: checkin.claimed_days,
        })
        .collect();
    let battle_pass = std::mem::take(&mut record.battle_pass);
    player.battle_pass = BattlePassState {
        id: battle_pass.id,
        paid_status: battle_pass.paid_status,
        level: battle_pass.level,
        exp: battle_pass.exp,
        exp_week: battle_pass.exp_week,
        week: battle_pass.week,
        rewards: battle_pass.rewards.into_iter().collect(),
        tasks: battle_pass.tasks,
    };
    player.mall_purchases = std::mem::take(&mut record.mall_purchases)
        .into_iter()
        .map(|purchase| MallPurchaseState {
            goods_id: purchase.goods_id,
            bought: purchase.bought,
            period: purchase.period,
        })
        .collect();
    player.recharge_purchases = std::mem::take(&mut record.recharge_purchases)
        .into_iter()
        .map(|purchase| RechargePurchaseState {
            recharge_id: purchase.recharge_id,
            purchase_count: purchase.purchase_count,
            last_bought_at: purchase.last_bought_at,
            expires_at: purchase.expires_at,
        })
        .collect();
    let boss_rush = std::mem::take(&mut record.boss_rush);
    player.boss_rush = BossRushState {
        id: boss_rush.id,
        ports: boss_rush
            .ports
            .into_iter()
            .map(|port| DcNetDataBossRushBase {
                pid: port.pid,
                role_ids: port.role_ids,
                damage: port.damage,
            })
            .collect(),
        rewards: boss_rush.rewards,
        ranking_rewards: boss_rush.ranking_rewards,
    };
    let activity_boss = std::mem::take(&mut record.activity_boss);
    player.activity_boss = ActivityBossState {
        aid: activity_boss.aid,
        day: activity_boss.day,
        damage: activity_boss.damage,
        daily_damage: activity_boss.daily_damage,
        role_ids: activity_boss.role_ids,
        rewards: activity_boss.rewards,
        daily_rewards: activity_boss.daily_rewards.into_iter().collect(),
    };
    let rift = std::mem::take(&mut record.rift);
    player.rift = RiftState {
        id: rift.id,
        current_port_id: rift.current_port_id,
        best_buff_score: rift.best_buff_score,
        best_score: rift.best_score,
        best_stage: rift.best_stage,
        completion_count: rift.completion_count,
        run_buff_score: rift.run_buff_score,
        run_score: rift.run_score,
        run_duration: rift.run_duration,
        active: rift.active,
        roles: rift.roles,
        tasks: rift.tasks,
    };
    player.rouge_progress = std::mem::take(&mut record.rouge_progress)
        .into_iter()
        .map(|progress| (progress.id, progress.pass))
        .collect();
    player.rouge_technology = std::mem::take(&mut record.rouge_technology);
    player.rouge_score = std::mem::take(&mut record.rouge_score);
    player.rouge_score_point_at = std::mem::take(&mut record.rouge_score_point_at);
    player.rouge_run = std::mem::take(&mut record.rouge_run).map(|data| {
        <DcNetDataRouge as protocol::prost::Message>::decode(data.as_slice())
            .expect("stored Rogue run is valid protobuf")
    });
}
