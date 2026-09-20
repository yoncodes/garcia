use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn gold_coin_rewards_are_table_driven_and_single_claim() {
    let (mut tables, config) = fixture();
    tables.cultivation_constants.coin_item_id = 100_100_002;
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let reward = player
        .collect_gold_coin(140_101_056, "nh_tower_cion_8", &tables)
        .unwrap();
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (tables.cultivation_constants.coin_item_id, 500)
    );
    assert_eq!(player.gold_coin_ids(140_101_056), ["nh_tower_cion_8"]);
    assert_eq!(
        player.collect_gold_coin(140_101_000, "nh_tower_cion_8", &tables),
        Err(GoldCoinError::AlreadyCollected("nh_tower_cion_8".into()))
    );
}

#[test]
fn collection_resources_use_table_counts_rewards_and_depletion() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert_eq!(
        player
            .collection_resource_info(140_101_056, &tables)
            .get("nh_tower_collection_01"),
        Some(&2)
    );

    let first = player
        .collect_collection_resource("nh_tower_collection_01", &tables, 123)
        .unwrap();
    assert_eq!(
        (first.items[0].item_id, first.items[0].amount),
        (100_307_001, 1)
    );
    let second = player
        .collect_collection_resource("nh_tower_collection_01", &tables, 124)
        .unwrap();
    assert_eq!(second.items[0].amount, 2);
    assert_eq!(
        player
            .collection_resource_info(140_101_056, &tables)
            .get("nh_tower_collection_01"),
        Some(&0)
    );
    assert_eq!(
        player.collect_collection_resource("nh_tower_collection_01", &tables, 125),
        Err(CollectionResourceError::Depleted(
            "nh_tower_collection_01".into()
        ))
    );
}

#[test]
fn wild_monster_points_remove_once_reward_and_reset_daily() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let (groups, _) = player.monster_point_groups(140_101_053, &tables, 123);
    assert!(
        groups
            .iter()
            .find(|group| group.id == "nh_field_a_group_01")
            .unwrap()
            .points
            .contains(&"nh_field_a_group_01_p1".into())
    );

    let (reward, group, limits) = player
        .defeat_monster_point("nh_field_a_group_01_p1", &tables, 123)
        .unwrap();
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (100_100_201, 10)
    );
    assert!(!group.points.contains(&"nh_field_a_group_01_p1".into()));
    assert_eq!((limits[0].num, limits[0].limit), (10, 1_200));
    assert_eq!(player.region_coin_amount(100_100_201), 10);
    let (duplicate_reward, duplicate_group, duplicate_limits) = player
        .defeat_monster_point("nh_field_a_group_01_p1", &tables, 123)
        .unwrap();
    assert!(duplicate_reward.items.is_empty());
    assert!(
        !duplicate_group
            .points
            .contains(&"nh_field_a_group_01_p1".into())
    );
    assert_eq!(duplicate_limits[0].num, 10);
    assert_eq!(player.region_coin_amount(100_100_201), 10);

    player.region_coin_daily.insert(100_100_201, 1_195);
    let (reward, _, limits) = player
        .defeat_monster_point("nh_field_a_group_01_p2", &tables, 123)
        .unwrap();
    assert_eq!(reward.items[0].amount, 15);
    assert_eq!(limits[0].num, 1_200);
    assert!(limits[0].reach_limit);

    let (groups, reset) = player.monster_point_groups(140_101_053, &tables, 86_523);
    assert!(reset);
    assert!(
        groups
            .iter()
            .find(|group| group.id == "nh_field_a_group_01")
            .unwrap()
            .points
            .contains(&"nh_field_a_group_01_p1".into())
    );
    assert_eq!(player.region_coin_amount(100_100_201), 0);
}

#[test]
fn wild_monster_rewards_follow_the_monster_drop_table() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let (regular, _, regular_limits) = player
        .defeat_monster_point("nh_secret_passage_group03_p4", &tables, 123)
        .unwrap();
    let (elite, _, elite_limits) = player
        .defeat_monster_point("nh_secret_passage_group03_p3", &tables, 123)
        .unwrap();

    assert_eq!(regular.items[0].amount, 10);
    assert_eq!(regular_limits[0].num, 10);
    assert_eq!(elite.items[0].amount, 60);
    assert_eq!(elite_limits[0].num, 60);
}

#[test]
fn wild_monster_reward_never_reduces_an_over_cap_daily_total() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.monster_point_groups(140_101_053, &tables, 123);
    player.region_coin_daily.insert(100_100_201, 1_205);

    let (reward, _, limits) = player
        .defeat_monster_point("nh_field_a_group_01_p1", &tables, 123)
        .unwrap();

    assert!(reward.items.is_empty());
    assert_eq!(player.region_coin_amount(100_100_201), 1_205);
    assert_eq!((limits[0].num, limits[0].limit), (1_205, 1_200));
    assert!(limits[0].reach_limit);
}

#[test]
fn region_info_uses_daily_task_reputation_and_coin_tables() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_524_037;
    player.region_coin_daily.insert(100_100_201, 230);

    let info = player
        .region_info(3, &tables, now, config.server.zone_offset)
        .unwrap();
    assert_eq!((info.id, info.remain_sec, info.level), (3, 45_163, 1));
    assert_eq!(info.tasks.len(), 5);
    let offered_types: std::collections::HashSet<_> = info
        .tasks
        .iter()
        .filter_map(|task| {
            tables
                .region_tasks_by_region
                .get(3)
                .unwrap()
                .find(|row| row.task_group_id == task.id)
                .map(|row| row.task_type)
        })
        .collect();
    assert!(offered_types.len() >= 3);
    assert!(info.tasks.iter().all(|task| {
        task.status == TaskStatus::Disabled as i32
            && task.curr_task.is_none()
            && tables
                .region_tasks_by_region
                .get(3)
                .unwrap()
                .any(|row| row.task_group_id == task.id)
    }));
    assert_eq!(
        info.tasks,
        player
            .region_info(3, &tables, now, config.server.zone_offset)
            .unwrap()
            .tasks
    );
    let coin = info.region_coin.unwrap();
    assert_eq!(
        (coin.coin_id, coin.num, coin.limit),
        (100_100_201, 230, 1_200)
    );
    assert!(!coin.reach_limit);

    player.items.push(DcNetDataItem {
        item_id: 100_100_102,
        amount: 200,
        ..Default::default()
    });
    assert_eq!(
        player
            .region_info(3, &tables, now, config.server.zone_offset)
            .unwrap()
            .level,
        2
    );
    let inactive = player
        .region_info(2, &tables, now, config.server.zone_offset)
        .unwrap();
    assert!(inactive.tasks.is_empty());
    assert!(inactive.region_coin.unwrap().reach_limit);
}

#[test]
fn region_ticket_claim_and_task_acceptance_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_524_037;
    let zone_offset = config.server.zone_offset;
    player.add_item(100_100_016, 2, &tables).unwrap();

    let ticket = player
        .claim_region_daily_ticket(&tables, now, zone_offset)
        .unwrap();
    assert_eq!((ticket.item_id, ticket.amount), (100_100_016, 7));
    assert!(player.is_region_ticket_taken(&tables, now, zone_offset));
    assert_eq!(
        player.claim_region_daily_ticket(&tables, now, zone_offset),
        Err(RegionError::DailyTicketClaimed)
    );

    let group_id = player
        .region_info(3, &tables, now, zone_offset)
        .unwrap()
        .tasks[0]
        .id;
    let (group, remains) = player
        .accept_region_task(3, group_id, &tables, now, zone_offset)
        .unwrap();
    assert_eq!(group.status, TaskStatus::Picked as i32);
    assert_eq!(remains[0].amount, 6);
    let current = group.curr_task.unwrap();
    assert_eq!(tables.tasks.get(current.id).unwrap().task_group, group_id);
    assert_eq!(
        player
            .region_info(3, &tables, now, zone_offset)
            .unwrap()
            .tasks
            .into_iter()
            .find(|task| task.id == group_id)
            .unwrap()
            .curr_task,
        Some(current)
    );
    assert_eq!(
        player.accept_region_task(3, group_id, &tables, now, zone_offset),
        Err(RegionError::AlreadyAccepted(group_id))
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert!(restored.is_region_ticket_taken(&tables, now, zone_offset));
    assert_eq!(restored.tasks, player.tasks);
}

#[test]
fn region_task_rewards_roll_over_reputation_and_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_524_037;
    let zone_offset = config.server.zone_offset;
    player.add_item(100_100_016, 4, &tables).unwrap();
    let mut expected_money = 0;
    let groups = player
        .region_info(3, &tables, now, zone_offset)
        .unwrap()
        .tasks
        .into_iter()
        .take(4)
        .map(|task| task.id)
        .collect::<Vec<_>>();

    for group_id in groups {
        let (group, _) = player
            .accept_region_task(3, group_id, &tables, now, zone_offset)
            .unwrap();
        let mut task_id = group.curr_task.unwrap().id;
        loop {
            let outcome = player.complete_task(task_id, 0, &tables, now).unwrap();
            if let Some((region_id, exp_item)) = outcome.region_reward {
                assert_eq!(region_id, 3);
                assert_eq!(exp_item.item_id, 100_100_102);
                assert_eq!(outcome.rewards.items.len(), 1);
                assert_eq!(outcome.rewards.items[0].item_id, 100_100_201);
                expected_money = outcome.rewards.items[0].amount;
                break;
            }
            task_id = outcome.changed[0].id;
        }
    }

    let info = player.region_info(3, &tables, now, zone_offset).unwrap();
    assert_eq!(info.level, 2);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_100_102)
            .unwrap()
            .amount,
        0
    );
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_100_201)
            .unwrap()
            .amount,
        expected_money
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .region_info(3, &tables, now, zone_offset)
            .unwrap()
            .level,
        2
    );
}

#[test]
fn achievements_use_live_progress_table_rewards_and_single_claim() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let level = player
        .achievements(&tables, player.created_at)
        .into_iter()
        .find(|achievement| achievement.id == 100_301)
        .unwrap();
    assert_eq!((level.progress, level.total), (player.level, 10));

    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_240,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    let (achievement, reward) = player.claim_achievement(500_101, &tables, 123).unwrap();
    assert_eq!(
        (
            achievement.id,
            achievement.took_at,
            achievement.progress,
            achievement.total
        ),
        (500_101, 123, 1, 1)
    );
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (100_100_002, 20)
    );
    assert_eq!(
        player.claim_achievement(500_101, &tables, 124),
        Err(AchievementError::AlreadyClaimed(500_101))
    );
}

#[test]
fn achievements_track_album_selection_and_lifetime_currency_acquisition() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = player.created_at;
    let before = player.achievements(&tables, now);

    player.add_item(100_399_003, 1, &tables).unwrap();
    player.unlock_available_albums(&tables, now);
    assert!(player.select_albums(&[1_000, 2_013]).unwrap());

    let updates = player.completed_achievements_since(&before, &tables, now);
    assert!(updates.iter().any(|achievement| (
        achievement.id,
        achievement.progress,
        achievement.total
    ) == (700_201, 1, 1)));

    player.item_acquired.insert(100_100_003, 1_038_100);
    let gold = player
        .achievements(&tables, now)
        .into_iter()
        .find(|achievement| achievement.id == 200_702)
        .unwrap();
    assert_eq!((gold.progress, gold.total), (1_000_000, 1_000_000));
}

#[test]
fn equipment_ownership_achievement_uses_current_inventory() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.equips.clear();
    player.equips.push(DcNetDataEquip::default());

    let achievement = player
        .achievements(&tables, 123)
        .into_iter()
        .find(|achievement| achievement.id == 200_501)
        .unwrap();

    assert_eq!((achievement.progress, achievement.total), (1, 1));
}
