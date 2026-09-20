use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

fn runtime(config: &common::config::ServerConfig, now: i32) -> DungeonRuntime {
    DungeonRuntime {
        now,
        zone_offset: config.server.zone_offset,
        player_exp_per_stamina: config.gameplay.dungeon_player_exp_per_stamina,
    }
}

#[test]
fn dungeon_lifecycle_spends_cost_rewards_completion_and_sweeps() {
    let (tables, config) = fixture();
    let equipment_dungeon = tables.dungeon_ports.get(180_260_001).unwrap();
    assert_eq!(equipment_dungeon.displayed_rewards[0].key, 100_305_001);
    assert_eq!(equipment_dungeon.guaranteed_rewards[0].key, 100_300_008);
    assert_eq!(equipment_dungeon.possible_rewards[0].key, 100_305_001);

    let mut player = Player::new(42, &tables, &config.account_defaults);
    let id = 180_200_001;
    player.level = 8;
    let stamina_before = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_008)
        .unwrap()
        .amount;
    let gold_before = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_003)
        .map_or(0, |item| item.amount);
    player.level = 10;
    player.add_user_exp(720, &tables);
    player.daily_tasks = vec![
        DcNetDataTaskCycle {
            id: 7,
            total: 1,
            ..Default::default()
        },
        DcNetDataTaskCycle {
            id: 10,
            total: 3,
            ..Default::default()
        },
    ];

    player.start_dungeon(id, 2, &tables, 123).unwrap();
    let settled = player
        .settle_dungeon(
            &DcNetDataParamsSettlement {
                id,
                is_completed: 1,
                ..Default::default()
            },
            1,
            2,
            &tables,
            runtime(&config, 124),
        )
        .unwrap();
    assert_eq!(settled.gameplay_id, id);
    assert_eq!(
        (settled.remains[0].item_id, settled.remains[0].amount),
        (100_100_008, stamina_before - 40)
    );
    let heat_update = settled.heat_update.unwrap();
    assert_eq!(heat_update.item, settled.remains[0]);
    assert_eq!(
        heat_update.cd_time,
        tables.cultivation_constants.heat_regeneration_interval
    );
    assert_eq!(settled.rewards.items[0].item_id, 100_100_003);
    assert_eq!(settled.rewards.items[0].amount, gold_before + 199_000);
    let user_up_data = settled.user_up_data.unwrap();
    assert_eq!(user_up_data.attr_lv, 11);
    assert_eq!(user_up_data.add_exp, 400);
    assert_eq!(user_up_data.items[0].item_id, PLAYER_EXP_ITEM_ID);
    assert_eq!(user_up_data.items[0].amount, 160);
    assert!(user_up_data.is_lv_up);
    assert_eq!(player.completed_dungeons, [id]);
    assert_eq!(player.item_spent.get(&100_100_008), Some(&40));
    assert_eq!(player.dungeon_clears.get(&2), Some(&1));
    assert_eq!(player.condition_progress(41, "2|999", &tables, 124), 1);
    assert_eq!(
        player.condition_progress(51, "100100008|180", &tables, 124),
        40
    );
    assert!(
        settled
            .mission_updates
            .iter()
            .any(|mission| mission.misson_id == 11 && mission.curr_num == 11)
    );
    assert!(
        settled
            .achievement_updates
            .iter()
            .any(|achievement| achievement.id == 300_201 && achievement.progress == 1)
    );
    assert!(
        settled
            .seven_day_updates
            .iter()
            .any(|activity| activity.id == 1 && activity.progress == 1)
    );
    assert_eq!(
        settled
            .daily_updates
            .iter()
            .map(|task| (task.id, task.progress, task.total))
            .collect::<Vec<_>>(),
        [(7, 1, 1)]
    );

    for timestamp in [125, 126] {
        player.start_dungeon(id, 2, &tables, timestamp).unwrap();
        let settled = player
            .settle_dungeon(
                &DcNetDataParamsSettlement {
                    id,
                    is_completed: 1,
                    ..Default::default()
                },
                1,
                2,
                &tables,
                runtime(&config, timestamp),
            )
            .unwrap();
        assert!(settled.heat_update.is_none());
        if timestamp == 125 {
            assert!(settled.daily_updates.is_empty());
        } else {
            assert_eq!(
                settled
                    .daily_updates
                    .iter()
                    .map(|task| (task.id, task.progress, task.total))
                    .collect::<Vec<_>>(),
                [(10, 3, 3)]
            );
        }
    }
    assert_eq!(player.item_spent.get(&100_100_008), Some(&120));
    assert_eq!(player.dungeon_clears.get(&2), Some(&3));

    let swept = player
        .sweep_dungeon(
            id,
            2,
            &tables,
            127,
            config.gameplay.dungeon_player_exp_per_stamina,
        )
        .unwrap();
    assert!(swept.heat_update.is_none());
    assert_eq!(swept.infos.len(), 2);
    assert_eq!(swept.infos[0].user_up_data.as_ref().unwrap().add_exp, 400);
    assert_eq!(swept.infos[1].user_up_data.as_ref().unwrap().add_exp, 400);
    assert_eq!(swept.remains[0].amount, stamina_before - 200);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_100_003)
            .unwrap()
            .amount,
        gold_before + 995_000
    );
    assert_eq!(player.completed_dungeons, [id]);
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.completed_dungeons, [id]);
    assert_eq!(restored.item_spent.get(&100_100_008), Some(&200));
    assert_eq!(restored.dungeon_clears.get(&2), Some(&3));
    assert!(matches!(
        player.sweep_dungeon(
            180_200_002,
            1,
            &tables,
            128,
            config.gameplay.dungeon_player_exp_per_stamina,
        ),
        Err(DungeonError::NotCompleted(180_200_002))
    ));
}

#[test]
fn relic_dungeons_grant_one_possible_box_per_run() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let dungeon = tables.dungeon_ports.get(180_260_001).unwrap();
    let possible = dungeon
        .possible_rewards
        .iter()
        .map(|reward| reward.key)
        .collect::<Vec<_>>();

    let rewards = player.grant_dungeon_rewards(dungeon, 2, &tables, 123);
    assert_eq!(
        player
            .items
            .iter()
            .filter(|item| possible.contains(&item.item_id))
            .map(|item| item.amount)
            .sum::<i32>(),
        2
    );
    assert!(rewards.items.iter().all(|item| {
        possible.contains(&item.item_id)
            || dungeon
                .guaranteed_rewards
                .iter()
                .any(|reward| reward.key == item.item_id)
    }));
}

#[test]
fn mixed_relic_dungeons_use_server_quality_weights() {
    let (tables, _) = fixture();
    let blue_purple = tables.dungeon_ports.get(180_260_002).unwrap();

    assert_eq!(
        select_relic_quality(&blue_purple.possible_rewards, &tables, 0),
        Some(4)
    );
    assert_eq!(
        select_relic_quality(&blue_purple.possible_rewards, &tables, 69),
        Some(4)
    );
    assert_eq!(
        select_relic_quality(&blue_purple.possible_rewards, &tables, 70),
        Some(5)
    );
    assert_eq!(
        select_relic_quality(&blue_purple.possible_rewards, &tables, 94),
        Some(5)
    );

    let purple_gold = tables.dungeon_ports.get(180_260_004).unwrap();
    assert_eq!(
        select_relic_quality(&purple_gold.possible_rewards, &tables, 0),
        Some(5)
    );
    assert_eq!(
        select_relic_quality(&purple_gold.possible_rewards, &tables, 24),
        Some(5)
    );
    assert_eq!(
        select_relic_quality(&purple_gold.possible_rewards, &tables, 25),
        Some(6)
    );
    assert_eq!(
        select_relic_quality(&purple_gold.possible_rewards, &tables, 29),
        Some(6)
    );
}

#[test]
fn higher_dungeon_tiers_require_the_table_world_level() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let dungeon_id = 180_200_002;
    player.level = 22;

    assert_eq!(
        player.start_dungeon(dungeon_id, 2, &tables, 123),
        Err(DungeonError::Locked(dungeon_id))
    );

    let world_level_task = tables
        .world_levels
        .iter()
        .find(|world_level| world_level.level == 1)
        .unwrap()
        .task_id;
    player.tasks.push(DcNetDataTaskStatus {
        id: world_level_task,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });

    player.start_dungeon(dungeon_id, 2, &tables, 124).unwrap();
}

#[test]
fn dungeon_settlement_reports_table_driven_battle_pass_progress() {
    let (tables, config) = fixture();
    let now = 1_787_587_200;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 10;
    let id = 180_200_001;

    player.start_dungeon(id, 2, &tables, now).unwrap();
    let settled = player
        .settle_dungeon(
            &DcNetDataParamsSettlement {
                id,
                is_completed: 1,
                ..Default::default()
            },
            1,
            2,
            &tables,
            runtime(&config, now),
        )
        .unwrap();

    assert!(
        settled
            .battle_pass_updates
            .iter()
            .any(|task| { task.id == 10_032_010 && (task.progress, task.total) == (1, 5) })
    );
    assert!(
        settled
            .battle_pass_updates
            .iter()
            .any(|task| { task.id == 10_031_030 && (task.progress, task.total) == (40, 180) })
    );
}
