use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn draw_history_uses_actual_rewards_filters_groups_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.add_item(100_100_024, 2, &tables).unwrap();

    player
        .draw_gacha(2, 1, &tables, 1_787_523_760, config.server.zone_offset)
        .unwrap();
    player
        .draw_gacha(2, 1, &tables, 1_787_523_761, config.server.zone_offset)
        .unwrap();

    let group_id = tables.gacha_pools.get(2).unwrap().group_id;
    let logs = player.gacha_logs(group_id, &tables);
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].time, 1_787_523_761);
    assert_eq!(logs[1].time, 1_787_523_760);
    assert_eq!(logs[1].reward, 110_100_010);
    assert!(player.gacha_logs(i32::MIN, &tables).is_empty());

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.gacha_logs(group_id, &tables), logs);
}

#[test]
fn cumulative_draw_reward_unlocks_claims_and_updates_gacha_state() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let required = tables.cultivation_constants.new_pool_cumulative_draw_target;
    assert_eq!(required, 50);
    player.add_item(100_100_024, required, &tables).unwrap();

    for offset in 0..required / 10 {
        player
            .draw_gacha(
                2,
                10,
                &tables,
                1_787_523_760 + offset,
                config.server.zone_offset,
            )
            .unwrap();
    }
    assert_eq!(
        (player.gachas[0].reward_cnt, player.gachas[0].reward_num),
        (0, 1)
    );
    assert!(player.gachas[0].taken_new_reward);

    let selected = tables
        .cultivation_constants
        .cumulative_draw_rewards
        .iter()
        .copied()
        .find(|id| !player.owns_role(*id))
        .unwrap();
    let outcome = player
        .claim_gacha_cumulative(selected, &tables, 1_787_523_800, config.server.zone_offset)
        .unwrap();
    assert_eq!(outcome.reward.role_rewards.len(), 1);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GachaEvent::ProfileAvatar(avatar)
            if tables.profile_avatar_for_maid(selected).is_some_and(|row| row.id == avatar.id)
    )));
    assert_eq!(
        (
            outcome.gacha_data.gacha_id,
            outcome.gacha_data.reward_num,
            outcome.gacha_data.taken_new_reward,
            outcome.gacha_data.had_take_reward,
            outcome.gacha_data.mall_goods.len(),
        ),
        (2, 0, true, true, 10)
    );
    assert!(matches!(
        player.claim_gacha_cumulative(selected, &tables, 1_787_523_801, config.server.zone_offset,),
        Err(GachaError::CumulativeRewardUnavailable)
    ));

    let limited = tables.gacha_pools.get(8).unwrap();
    player
        .add_item(
            limited.single_draw_cost.key,
            limited.single_draw_cost.value * 10,
            &tables,
        )
        .unwrap();
    let limited_outcome = player
        .draw_gacha(8, 10, &tables, 1_787_523_802, config.server.zone_offset)
        .unwrap();
    assert_eq!(
        (
            limited_outcome.gacha_data.all_count,
            limited_outcome.gacha_data.reward_cnt,
            limited_outcome.gacha_data.reward_num,
            limited_outcome.gacha_data.taken_new_reward,
        ),
        (10, 0, 0, false)
    );
    assert_eq!(
        limited_outcome
            .reward_list
            .iter()
            .map(|draw| draw.reward_res.unwrap().idx)
            .collect::<Vec<_>>(),
        (0..10).collect::<Vec<_>>()
    );
}
#[test]
fn gacha_matches_captured_first_draw_and_persists_owned_role() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let zone_offset = config.server.zone_offset;
    let list = player.gacha_pools(&tables, 1_787_523_756, zone_offset);
    assert_eq!(
        list.iter().map(|gacha| gacha.gacha_id).collect::<Vec<_>>(),
        [7, 8, 2]
    );
    assert_eq!(
        list.iter()
            .map(|gacha| gacha.remain_sec)
            .collect::<Vec<_>>(),
        [304_645, 2_723_845, 0]
    );
    assert_eq!(
        list.iter()
            .map(|gacha| gacha.mall_goods.len())
            .collect::<Vec<_>>(),
        [6, 6, 10]
    );

    player.add_item(100_100_024, 1, &tables).unwrap();
    let outcome = player
        .draw_gacha(2, 1, &tables, 1_787_523_760, zone_offset)
        .unwrap();
    let draw = &outcome.reward_list[0];
    let reward = draw.reward_res.unwrap();
    assert_eq!(
        (draw.gacha_id, draw.reward_id, draw.hit_10, draw.hit_s),
        (2, 130_200_201, true, false)
    );
    assert_eq!(
        (
            reward.reward,
            reward.amount,
            reward.quality,
            reward.duplicate
        ),
        (110_100_010, 1, 5, 0)
    );
    assert_eq!(
        (outcome.remains[0].item_id, outcome.remains[0].amount),
        (100_100_024, 0)
    );
    assert_eq!(
        outcome.coin.map(|coin| (coin.item_id, coin.amount)),
        Some((100_100_023, 1))
    );
    assert_eq!(
        (
            outcome.gacha_data.all_count,
            outcome.gacha_data.reward_cnt,
            outcome.gacha_data.gacha_count_10
        ),
        (1, 1, 0)
    );
    assert_eq!(outcome.battle_pass_updates.len(), 1);
    assert_eq!(
        (
            outcome.battle_pass_updates[0].id,
            outcome.battle_pass_updates[0].progress,
            outcome.battle_pass_updates[0].total,
        ),
        (10_033_030, 1, 40)
    );
    assert!(player.owns_role(110_100_010));
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GachaEvent::Archive(archive) if archive.id == 1002
    )));
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GachaEvent::ProfileAvatar(avatar) if avatar.id == 101_000_010
    )));

    let mut mixed_payment = Player::new(43, &tables, &config.account_defaults);
    let ticket_id = tables.gacha_pools.get(2).unwrap().single_draw_cost.key;
    mixed_payment.add_item(ticket_id, 9, &tables).unwrap();
    let diamond_id = tables.cultivation_constants.diamond_item_id;
    let diamonds_before = mixed_payment
        .add_item(diamond_id, 160, &tables)
        .unwrap()
        .amount;
    let mixed_outcome = mixed_payment
        .draw_gacha(2, 10, &tables, 1_787_523_760, zone_offset)
        .unwrap();
    assert_eq!(
        mixed_outcome
            .remains
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(diamond_id, diamonds_before - 160), (ticket_id, 0)]
    );

    let duplicate = player
        .apply_gacha_reward(110_100_010, 1, &tables, 1_787_523_761)
        .unwrap();
    assert_eq!(
        (
            duplicate.reward_type,
            duplicate.reward,
            duplicate.duplicate,
            duplicate.amount,
            duplicate.quality
        ),
        (2, 110_100_010, 100_200_010, 1, 5)
    );

    player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|info| info.game_role_id == 110_100_010)
        })
        .unwrap()
        .maid_qua = tables.cultivation_constants.max_role_resonance - 1;
    let capped_duplicate = player
        .apply_gacha_reward(110_100_010, 1, &tables, 1_787_523_762)
        .unwrap();
    assert_eq!(
        (
            capped_duplicate.duplicate,
            capped_duplicate.amount,
            capped_duplicate.quality,
        ),
        (100_300_013, 10, 5)
    );
    let seven_day = player
        .seven_day_activity_status(&tables, player.created_at)
        .0
        .into_iter()
        .find(|activity| activity.id == 2)
        .unwrap();
    assert_eq!((seven_day.progress, seven_day.total), (1, 1));

    let record = player.to_record();
    let restored = Player::from_record(record, &tables, &config.account_defaults);
    assert!(restored.owns_role(110_100_010));
    assert_eq!(restored.gachas[0].all_count, 1);
}

#[test]
fn guaranteed_draw_uses_the_counter_since_the_last_maid() {
    let (tables, config) = fixture();
    let pool = tables.gacha_pools.get(2).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.gachas.push(GachaState {
        gacha_id: pool.id,
        all_count: 1,
        gacha_count_10: pool.guaranteed_draw_interval - 1,
        reward_cnt: 1,
        reward_num: 0,
        taken_new_reward: false,
        had_take_reward: false,
    });
    player
        .add_item(
            pool.single_draw_cost.key,
            pool.single_draw_cost.value,
            &tables,
        )
        .unwrap();

    let outcome = player
        .draw_gacha(
            pool.id,
            1,
            &tables,
            1_787_523_760,
            config.server.zone_offset,
        )
        .unwrap();

    assert_eq!(outcome.reward_list[0].reward_id, pool.ten_reward_id);
    assert_eq!(outcome.gacha_data.gacha_count_10, 0);
}

#[test]
fn gacha_categories_match_the_published_rates() {
    let (tables, _) = fixture();
    let pool = tables.gacha_pools.get(2).unwrap();

    assert_eq!(
        select_gacha_category(pool, false, &tables, 119),
        Some(GachaCategory::Quality6Maid)
    );
    assert_eq!(
        select_gacha_category(pool, false, &tables, 120),
        Some(GachaCategory::Quality5Maid)
    );
    assert_eq!(
        select_gacha_category(pool, false, &tables, 619),
        Some(GachaCategory::Quality5Maid)
    );
    assert_eq!(
        select_gacha_category(pool, false, &tables, 620),
        Some(GachaCategory::Quality5Partner)
    );
    assert_eq!(
        select_gacha_category(pool, false, &tables, 1_119),
        Some(GachaCategory::Quality5Partner)
    );
    assert_eq!(
        select_gacha_category(pool, false, &tables, 1_120),
        Some(GachaCategory::Quality4Partner)
    );
    assert_eq!(
        select_gacha_category(pool, true, &tables, 119),
        Some(GachaCategory::Quality6Maid)
    );
    assert_eq!(
        select_gacha_category(pool, true, &tables, 120),
        Some(GachaCategory::Quality5Maid)
    );
    assert_eq!(
        select_gacha_category(pool, true, &tables, 9_999),
        Some(GachaCategory::Quality5Maid)
    );
}

#[test]
fn limited_gacha_uses_a_fifty_percent_featured_split() {
    let (tables, _) = fixture();
    let pool = tables.gacha_pools.get(8).unwrap();
    let candidates = [110_100_012, 110_100_007];

    assert_eq!(
        select_gacha_candidate(pool, &candidates, 4_999, 0),
        Some(110_100_012)
    );
    assert_eq!(
        select_gacha_candidate(pool, &candidates, 5_000, 0),
        Some(110_100_007)
    );
}

#[test]
fn limited_gacha_pity_carries_between_event_banners_and_persists() {
    let (tables, config) = fixture();
    let old_pool = tables.gacha_pools.get(7).unwrap();
    let active_pool = tables.gacha_pools.get(8).unwrap();
    assert_eq!(old_pool.group_id, active_pool.group_id);

    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.gachas.push(GachaState {
        gacha_id: old_pool.id,
        all_count: 1,
        gacha_count_10: 9,
        reward_cnt: 0,
        reward_num: 0,
        taken_new_reward: false,
        had_take_reward: false,
    });
    player
        .add_item(
            active_pool.single_draw_cost.key,
            active_pool.single_draw_cost.value,
            &tables,
        )
        .unwrap();

    let outcome = player
        .draw_gacha(
            active_pool.id,
            1,
            &tables,
            1_787_523_760,
            config.server.zone_offset,
        )
        .unwrap();

    assert_eq!(outcome.reward_list[0].reward_id, active_pool.ten_reward_id);
    assert!(
        tables
            .maids
            .get(outcome.reward_list[0].reward_res.unwrap().reward)
            .is_some()
    );
    assert_eq!(outcome.gacha_data.gacha_count_10, 0);
    assert!(
        player
            .gachas
            .iter()
            .filter(|state| matches!(state.gacha_id, 7 | 8))
            .all(|state| state.gacha_count_10 == 0)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .gacha_pools(&tables, 1_787_523_760, config.server.zone_offset)
            .into_iter()
            .find(|gacha| gacha.gacha_id == active_pool.id)
            .unwrap()
            .gacha_count_10,
        0
    );
}

#[test]
fn partner_draws_unlock_archives_and_complete_captured_count_milestones() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let before = player.achievements(&tables, 1_787_523_760);

    let first_partner = player
        .apply_gacha_reward(100_444_900, 2, &tables, 1_787_523_760)
        .unwrap();
    assert_eq!(first_partner.reward_type, 8);
    player
        .apply_gacha_reward(100_442_900, 2, &tables, 1_787_523_760)
        .unwrap();
    player
        .apply_gacha_reward(100_445_900, 2, &tables, 1_787_523_760)
        .unwrap();

    assert_eq!(
        player
            .archive_infos(&tables, 1_787_523_760)
            .iter()
            .find(|archive| archive.id == 2018)
            .map(|archive| archive.in_time),
        Some(1_787_523_760)
    );
    assert_eq!(
        player.completed_achievements_since(&before, &tables, 1_787_523_760),
        [DcNetDataAchi {
            id: 200_301,
            progress: 3,
            total: 3,
            took_at: 0,
        }]
    );
}

#[test]
fn partner_draw_reuses_a_free_instance_id_without_colliding() {
    let (tables, config) = fixture();
    let partner_id = 100_444_900;
    let mut player = Player::new(42, &tables, &config.account_defaults);

    for _ in 0..3 {
        player
            .apply_gacha_reward(partner_id, 2, &tables, 1_787_523_760)
            .unwrap();
    }
    let removed_id = player.partners.remove(1).id;
    let replacement = player
        .apply_gacha_reward(partner_id, 2, &tables, 1_787_523_761)
        .unwrap();

    assert_eq!(replacement.user_dat_id, removed_id);
    assert_eq!(
        player
            .partners
            .iter()
            .map(|partner| partner.id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        player.partners.len()
    );
}

#[test]
fn only_maid_results_reset_the_ten_pull_counter() {
    let (mut tables, config) = fixture();
    let pool = tables
        .gacha_pools
        .rows
        .iter_mut()
        .find(|pool| pool.id == 2)
        .unwrap();
    pool.first_get = 100_400_026;
    let ticket_id = pool.single_draw_cost.key;

    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.add_item(ticket_id, 1, &tables).unwrap();
    player.gachas.push(GachaState {
        gacha_id: 2,
        all_count: 0,
        gacha_count_10: 5,
        reward_cnt: 0,
        reward_num: 0,
        taken_new_reward: false,
        had_take_reward: false,
    });
    let partner = player
        .draw_gacha(2, 1, &tables, 1_787_523_760, config.server.zone_offset)
        .unwrap();
    assert_eq!(tables.partners.get(100_400_026).unwrap().quality, 5);
    assert!(!partner.reward_list[0].hit_10);
    assert_eq!(partner.gacha_data.gacha_count_10, 6);

    tables
        .gacha_pools
        .rows
        .iter_mut()
        .find(|pool| pool.id == 2)
        .unwrap()
        .first_get = 110_100_019;
    let mut player = Player::new(43, &tables, &config.account_defaults);
    player.add_item(ticket_id, 1, &tables).unwrap();
    player.gachas.push(GachaState {
        gacha_id: 2,
        all_count: 0,
        gacha_count_10: 5,
        reward_cnt: 0,
        reward_num: 0,
        taken_new_reward: false,
        had_take_reward: false,
    });
    let maid = player
        .draw_gacha(2, 1, &tables, 1_787_523_760, config.server.zone_offset)
        .unwrap();
    assert!(maid.reward_list[0].hit_10);
    assert_eq!(maid.gacha_data.gacha_count_10, 0);
}

#[test]
fn inactive_banner_override_controls_both_listing_and_drawing() {
    let (tables, config) = fixture();
    let now = 1_787_523_760;
    let zone_offset = config.server.zone_offset;
    let expired = tables.gacha_pools.get(3).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert!(
        player
            .gacha_pools(&tables, now, zone_offset)
            .iter()
            .all(|banner| banner.gacha_id != expired.id)
    );
    assert!(matches!(
        player.draw_gacha(expired.id, 1, &tables, now, zone_offset),
        Err(GachaError::Inactive(3))
    ));

    let expires_at = now + 30 * 86_400;
    let enabled =
        player.gacha_pools_with_overrides(&tables, now, zone_offset, &[(expired.id, expires_at)]);
    let banner = enabled
        .iter()
        .find(|banner| banner.gacha_id == expired.id)
        .unwrap();
    assert_eq!(banner.remain_sec, 30 * 86_400);

    player
        .add_item(
            expired.single_draw_cost.key,
            expired.single_draw_cost.value,
            &tables,
        )
        .unwrap();
    let outcome = player
        .draw_gacha_with_override(expired.id, 1, &tables, now, zone_offset, Some(expires_at))
        .unwrap();
    assert_eq!(outcome.gacha_data.gacha_id, expired.id);
    assert_eq!(outcome.gacha_data.remain_sec, 30 * 86_400);
    assert!(
        player
            .gacha_pools_with_overrides(
                &tables,
                expires_at,
                zone_offset,
                &[(expired.id, expires_at)],
            )
            .iter()
            .all(|banner| banner.gacha_id != expired.id)
    );
}
