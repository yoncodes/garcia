use super::*;

#[test]
fn limited_level_activity_uses_live_window_table_rewards_and_persistence() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_522_872;

    assert!(player.is_limited_level_activity_available(6, &tables, now, config.server.zone_offset));
    assert!(!player.is_limited_level_activity_available(
        1,
        &tables,
        now,
        config.server.zone_offset
    ));

    player.level = 20;
    let reward = player
        .claim_limited_level_reward(1, &tables, now, config.server.zone_offset)
        .unwrap();
    assert_eq!(reward.items.len(), 9);
    assert_eq!(
        reward
            .items
            .iter()
            .find(|item| item.item_id == 100_100_002)
            .unwrap()
            .amount,
        60
    );
    assert!(
        player
            .claim_limited_level_reward(1, &tables, now, config.server.zone_offset)
            .is_err()
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.activity_limited_level_rewards, [1]);
}

#[test]
fn version_activity_uses_unlock_time_tasks_rewards_and_persistence() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_587_200;

    assert_eq!(
        player.active_version_activity(&tables, now, config.server.zone_offset),
        None
    );
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_001_060,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    player.item_spent.insert(100_100_008, 240);
    assert_eq!(
        player
            .active_version_activity(&tables, now, config.server.zone_offset)
            .unwrap()
            .0,
        1003
    );
    let tasks = player.version_tasks(&tables, now, config.server.zone_offset);
    assert_eq!(tasks[0].id, 10_030_101);
    assert_eq!((tasks[0].progress, tasks[0].total), (240, 240));

    let (task, reward) = player
        .claim_version_task(10_030_101, &tables, now, config.server.zone_offset)
        .unwrap();
    assert_eq!(task.status, TaskStatus::Done as i32);
    assert_eq!(reward.items[0].item_id, 100_100_303);
    assert!(matches!(
        player.claim_version_task(10_030_101, &tables, now, config.server.zone_offset),
        Err(VersionTaskError::AlreadyClaimed(10_030_101))
    ));

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.version_task_claims, [10_030_101]);
}
fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn activity_level_reward_matches_capture_and_persists_once() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 10;
    let before = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_002)
        .map_or(0, |item| item.amount);

    let reward = player.claim_activity_level_reward(1, &tables, 123).unwrap();
    assert_eq!(
        reward
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_100_002, before + 300), (100_100_024, 10)]
    );
    assert!(player.claim_activity_level_reward(1, &tables, 124).is_err());

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.activity_level_rewards, [1]);
}

#[test]
fn seven_day_activity_unlocks_claims_and_uses_point_milestones() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = player.created_at;
    for id in ["box-1", "box-2", "box-3"] {
        player.reward_boxes.push(DcNetDataTBoxInfo {
            id: id.into(),
            status: 1,
        });
    }

    let (list, point_claimed) = player.seven_day_activity_status(&tables, now);
    assert_eq!(
        list.iter().map(|task| task.id).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert!(point_claimed.is_empty());
    let boxes = list.iter().find(|task| task.id == 3).unwrap();
    assert_eq!((boxes.progress, boxes.total), (3, 3));

    let (task, reward) = player
        .claim_seven_day_activity_reward(3, &tables, 123)
        .unwrap();
    assert_eq!((task.id, task.took_at), (3, 123));
    assert_eq!(
        reward
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_100_017, 1), (100_302_002, 20)]
    );
    assert_eq!(
        player.claim_seven_day_activity_reward(3, &tables, 124),
        Err(Activity7DayError::AlreadyClaimed(3))
    );

    player
        .items
        .iter_mut()
        .find(|item| item.item_id == 100_100_017)
        .unwrap()
        .amount = 4;
    let point = player
        .claim_seven_day_activity_milestone(1, &tables, 125)
        .unwrap();
    assert_eq!(
        point
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_100_024, 5), (100_100_002, 200)]
    );
    assert_eq!(player.activity_7day_point_claims, [1]);
    assert_eq!(
        player.claim_seven_day_activity_milestone(1, &tables, 126),
        Err(Activity7DayError::AlreadyClaimed(1))
    );
}

#[test]
fn checkin_matches_captured_activity_windows_rewards_and_daily_progress() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_522_879;
    let zone_offset = config.server.zone_offset;

    let activities = player.open_activities(&tables, now, zone_offset);
    assert_eq!(
        activities
            .iter()
            .map(|activity| activity.aid)
            .collect::<Vec<_>>(),
        [1, 3, 4, 6, 8, 9, 21, 22, 1003]
    );
    assert_eq!(
        activities
            .iter()
            .find(|activity| activity.aid == 22)
            .unwrap()
            .remain_time,
        46_320
    );
    assert_eq!(
        activities
            .iter()
            .find(|activity| activity.aid == 1003)
            .unwrap()
            .remain_time,
        305_521
    );

    let (checkins, changed) = player.check_in_status(&tables, now, zone_offset);
    assert!(changed);
    assert_eq!(
        checkins
            .iter()
            .map(|checkin| (checkin.cur_aid, checkin.check_days))
            .collect::<Vec<_>>(),
        [(1, 1), (1003, 1)]
    );
    player.add_item(100_100_002, 50, &tables).unwrap();
    let permanent = player
        .claim_check_in_reward(1, 1, &tables, now, zone_offset)
        .unwrap();
    assert_eq!(
        (permanent.items[0].item_id, permanent.items[0].amount),
        (100_100_002, 170)
    );
    let event = player
        .claim_check_in_reward(1003, 1, &tables, now, zone_offset)
        .unwrap();
    assert_eq!(
        (event.items[0].item_id, event.items[0].amount),
        (100_100_002, 250)
    );
    assert_eq!(
        player.claim_check_in_reward(1003, 1, &tables, now, zone_offset),
        Err(CheckinError::AlreadyClaimed { aid: 1003, day: 1 })
    );

    let (next_day, changed) = player.check_in_status(&tables, now + 86_400, zone_offset);
    assert!(changed);
    assert!(next_day.iter().all(|checkin| checkin.check_days == 2));
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.checkins, player.checkins);
}

#[test]
fn duplicate_maid_checkin_reward_includes_the_owned_role() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_789_663_200;
    let zone_offset = config.server.zone_offset;
    let maid_id = 110_100_002;
    let role = gacha_role(player.uid, maid_id, &tables).unwrap();
    player.roles.retain(|owned| {
        owned
            .role_basic_info
            .as_ref()
            .is_none_or(|info| info.game_role_id != maid_id)
    });
    player.roles.push(role.clone());

    player.check_in_status(&tables, now, zone_offset);
    player.check_in_status(&tables, now + 86_400, zone_offset);
    player.check_in_status(&tables, now + 172_800, zone_offset);
    let reward = player
        .claim_check_in_reward(1, 3, &tables, now + 172_800, zone_offset)
        .unwrap();

    assert_eq!(reward.role_rewards.len(), 1);
    assert_eq!(reward.role_rewards[0].role_info, Some(role));
    assert_eq!(
        reward.role_rewards[0]
            .on_duplicate
            .as_ref()
            .map(|item| item.item_id),
        Some(100_200_002)
    );
}

#[test]
fn quality_conditions_count_owned_entities_at_or_above_the_threshold() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.roles.clear();
    player.roles.push(DcNetDataRolesRolesDetail {
        role_basic_info: Some(DcNetDataRoleBasicInfo {
            qua: 6,
            ..Default::default()
        }),
        ..Default::default()
    });
    player.partners.clear();
    player.partners.push(DcNetDataPartner {
        lv: 12,
        quality: 6,
        ..Default::default()
    });
    player.equips.clear();
    player.equips.push(DcNetDataEquip {
        level: 20,
        quality: 7,
        ..Default::default()
    });

    assert_eq!(player.condition_progress(25, "1|20", &tables, 123), 1);
    assert_eq!(player.condition_progress(26, "1|12", &tables, 123), 1);
    assert_eq!(player.condition_progress(28, "6|999", &tables, 123), 1);
    assert_eq!(player.condition_progress(201, "6|999", &tables, 123), 1);
    assert_eq!(player.condition_progress(203, "6|999", &tables, 123), 1);
    assert_eq!(player.condition_progress(203, "8|999", &tables, 123), 0);
    assert_eq!(player.condition_progress(104, "999", &tables, 123), 20);
    assert_eq!(condition_total(25, "1|20"), 1);
    assert_eq!(condition_total(25, "3|30"), 3);
    assert_eq!(condition_total(26, "5|12"), 5);
}
