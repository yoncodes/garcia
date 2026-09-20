use super::*;

fn tables() -> GameTables {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    GameTables::load(&versions).unwrap()
}

#[test]
fn activity_boss_tracks_best_runs_rewards_days_and_persistence() {
    let tables = tables();
    let config = common::load_config().unwrap();
    let event = tables.activities.get(7).unwrap();
    let open = common::time::table_time_utc(&event.open_time, config.server.zone_offset).unwrap();
    let now = (open + 60) as i32;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_ids = player
        .roles
        .iter()
        .take(2)
        .map(|role| role.role_basic_info.as_ref().unwrap().game_role_id)
        .collect::<Vec<_>>();

    let (gameplay_id, score) = player
        .settle_activity_boss(
            7,
            12_000,
            role_ids.clone(),
            &tables,
            now,
            config.server.zone_offset,
        )
        .unwrap();
    assert_eq!((gameplay_id, score), (140_500_000, 12_000));
    assert_eq!(player.activity_boss.daily_rewards.get(&1), Some(&0));

    let point_reward = player
        .claim_activity_boss_reward(7, 1, &tables, now)
        .unwrap();
    let daily_reward = player
        .claim_activity_boss_daily_rewards(7, &[1], &tables, now)
        .unwrap();
    assert!(!point_reward.items.is_empty());
    assert!(!daily_reward.items.is_empty());
    assert_eq!(player.activity_boss.daily_rewards.get(&1), Some(&1));
    assert_eq!(
        player.claim_activity_boss_reward(7, 1, &tables, now),
        Err(ActivityBossError::RewardAlreadyClaimed(1))
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.activity_boss, player.activity_boss);

    let mut next_day = restored;
    let (info, can_fight, day, changed) = next_day
        .activity_boss_info(7, &tables, now + 86_400, config.server.zone_offset)
        .unwrap();
    assert!(can_fight);
    assert!(changed);
    assert_eq!(day, 2);
    assert_eq!(info.daily_damage, 0);
    assert_eq!(info.damage, 12_000);
    assert_eq!(info.ids, [1]);
    assert_eq!(info.daily_rewards.get(&1), Some(&1));
}

#[test]
fn activity_boss_rejects_inactive_runs_and_unknown_roles() {
    let tables = tables();
    let config = common::load_config().unwrap();
    let event = tables.activities.get(7).unwrap();
    let open = common::time::table_time_utc(&event.open_time, config.server.zone_offset).unwrap();
    let close = common::time::table_time_utc(&event.close_time, config.server.zone_offset).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert_eq!(
        player.settle_activity_boss(
            7,
            1,
            Vec::new(),
            &tables,
            close as i32,
            config.server.zone_offset,
        ),
        Err(ActivityBossError::Inactive(7))
    );
    assert_eq!(
        player.settle_activity_boss(
            7,
            1,
            vec![-1],
            &tables,
            (open + 1) as i32,
            config.server.zone_offset,
        ),
        Err(ActivityBossError::UnknownRole(-1))
    );
}
