use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    (tables, config)
}

#[test]
fn matching_interaction_completes_the_active_task() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_140,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    player.set_interact("prologue_lift_move_mechanism", 1);
    let outcomes = player
        .advance_interact_tasks("prologue_lift_move_mechanism", 1, &tables, 123)
        .unwrap();

    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        (outcomes[0].progress.progress, outcomes[0].progress.total),
        (1, 1)
    );
    let completion = outcomes[0].completion.as_ref().unwrap();
    assert_eq!(completion.current.status, TaskStatus::Done as i32);
    assert!(
        completion
            .changed
            .iter()
            .any(|task| task.id == 1_500_001_150 && task.status == TaskStatus::Picked as i32)
    );
}

#[test]
fn resource_dungeon_completion_advances_the_region_task() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 170_101_400,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });
    player.completed_dungeons.push(180_200_001);

    let outcomes = player
        .advance_dungeon_tasks(180_200_001, &tables, 123)
        .unwrap();

    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        (outcomes[0].progress.progress, outcomes[0].progress.total),
        (1, 1)
    );
    assert_eq!(
        outcomes[0].completion.as_ref().unwrap().current.status,
        TaskStatus::Done as i32
    );
}

#[test]
fn reaching_a_required_player_level_completes_the_active_task() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 21;
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_009_000,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcomes = player.advance_level_tasks(&tables, 123).unwrap();

    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        (outcomes[0].progress.progress, outcomes[0].progress.total),
        (21, 21)
    );
    assert_eq!(
        outcomes[0].completion.as_ref().unwrap().current.status,
        TaskStatus::Done as i32
    );
}

#[test]
fn level_missions_only_update_when_their_threshold_is_crossed() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    assert!(
        player
            .missions
            .iter()
            .find(|mission| mission.misson_id == 1)
            .unwrap()
            .created_at
            > 0
    );

    player.level = 3;
    let updates = player.advance_level_missions(123);
    assert_eq!(
        updates
            .iter()
            .map(|mission| (mission.misson_id, mission.curr_num, mission.created_at))
            .collect::<Vec<_>>(),
        [(3, 3, 123), (2, 3, 123)]
    );

    player.level = 4;
    let updates = player.advance_level_missions(124);
    assert_eq!(updates.len(), 1);
    assert_eq!((updates[0].misson_id, updates[0].curr_num), (4, 4));
}

#[test]
fn completed_tasks_unlock_table_defined_features_and_rebuild_them_on_load() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_150,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcome = player
        .complete_task(1_500_001_150, 0, &tables, 123)
        .unwrap();
    assert_eq!(
        outcome
            .feat_updates
            .iter()
            .map(|feat| feat.id)
            .collect::<Vec<_>>(),
        [19, 84]
    );
    assert!(
        outcome
            .feat_updates
            .iter()
            .all(|feat| feat.status == FeatStatus::FeatUnlocked as i32)
    );
    assert_eq!(
        player.unlock_feature(19, &tables).unwrap().status,
        FeatStatus::FeatUnlocked as i32
    );
    assert_eq!(
        player.unlock_feature(i32::MAX, &tables),
        Err(FeatureError::NotUnlockable(i32::MAX))
    );

    player
        .feats
        .iter_mut()
        .find(|feat| feat.id == 84)
        .unwrap()
        .status = FeatStatus::FeatUnlockable as i32;
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert!(
        restored
            .feats
            .iter()
            .any(|feat| { feat.id == 19 && feat.status == FeatStatus::FeatUnlocked as i32 })
    );
    assert!(
        restored
            .feats
            .iter()
            .any(|feat| { feat.id == 84 && feat.status == FeatStatus::FeatUnlocked as i32 })
    );
}

#[test]
fn bulk_unlock_enables_every_active_feature_once() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let expected = tables
        .feature_unlocks
        .rows
        .iter()
        .filter(|feature| feature.enabled != 0)
        .count();

    let outcome = player.unlock_all_features(&tables, 123).unwrap();
    assert_eq!(outcome.features.len(), expected);
    assert!(!outcome.requirements.is_empty());
    for feature in tables
        .feature_unlocks
        .rows
        .iter()
        .filter(|feature| feature.enabled != 0)
    {
        for requirement in &feature.conditions {
            let task_id = requirement.value.parse::<i32>().unwrap();
            let task = player.tasks.iter().find(|task| task.id == task_id).unwrap();
            if requirement.key == 81 {
                assert_eq!(task.status, TaskStatus::Done as i32);
            }
        }
    }
    let repeated = player.unlock_all_features(&tables, 124).unwrap();
    assert!(repeated.requirements.is_empty());
    assert!(repeated.features.is_empty());
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .feats
            .iter()
            .filter(|feature| feature.status == FeatStatus::FeatUnlocked as i32)
            .count(),
        expected
    );
}

#[test]
fn completing_a_task_unlocks_its_interaction() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_130,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcome = player
        .complete_task(1_500_001_130, 0, &tables, 123)
        .unwrap();

    assert!(outcome.interact_updates.iter().any(|object| {
        object.object_id == "prologue_lift_move_mechanism" && object.interactive
    }));
}

#[test]
fn completing_a_task_unlocks_its_table_defined_album() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_005_050,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcome = player
        .complete_task(1_501_005_050, 0, &tables, 123)
        .unwrap();

    assert!(
        outcome
            .rewards
            .items
            .iter()
            .any(|item| { item.item_id == 100_399_003 && item.amount == 1 })
    );
    assert_eq!(
        outcome.album_updates,
        [DcNetDataAlbums {
            id: 2013,
            status: AlbumStatus::Unlocked as i32,
            sort: 0,
        }]
    );
    assert_eq!(player.unlock_available_albums(&tables, 123), []);
}

#[test]
fn completing_a_task_pushes_the_newly_unlocked_sms() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.extend(
        [1_501_005_020, 1_501_005_030, 1_501_005_040]
            .into_iter()
            .map(|id| DcNetDataTaskStatus {
                id,
                status: if id == 1_501_005_040 {
                    TaskStatus::Picked as i32
                } else {
                    TaskStatus::Done as i32
                },
                ..Default::default()
            }),
    );

    let outcome = player
        .complete_task(1_501_005_040, 0, &tables, 123)
        .unwrap();

    assert_eq!(
        outcome.sms_updates,
        [DcNetDataSms {
            id: 410_101,
            read: false,
            selected: Vec::new(),
        }]
    );
}

#[test]
fn manually_traced_task_group_validates_clears_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_140,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_150,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    assert_eq!(
        player.set_traced_task_group(11_000_101, &tables),
        Ok(11_000_101)
    );
    assert_eq!(player.traced_task_group, 11_000_101);
    assert_eq!(player.current_traced_task_ids(&tables), [1_500_000_000]);
    assert_eq!(
        player.set_traced_task_group(99, &tables),
        Err(TaskGoalError::Inactive(99))
    );
    assert_eq!(player.traced_task_group, 11_000_101);

    let mut restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.traced_task_group, 11_000_101);
    assert_eq!(restored.set_traced_task_group(0, &tables), Ok(0));
    assert_eq!(restored.traced_task_group, 0);
}
#[test]
fn mission_claim_uses_table_rewards_once() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let (mission, rewards) = player.claim_mission(1, &tables, 123).unwrap();
    assert!(mission.taken);
    assert_eq!(mission.take_time, 123);
    assert_eq!(rewards.items.len(), 3);
    assert_eq!(
        rewards
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_300_001, 1), (100_300_004, 1), (100_100_003, 7_000)]
    );
    assert_eq!(
        player.claim_mission(1, &tables, 124),
        Err(MissionClaimError::AlreadyClaimed(1))
    );
}

#[test]
fn completing_task_picks_configured_next_tasks() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let outcome = player
        .complete_task(1_500_000_000, 0, &tables, 123)
        .unwrap();
    assert_eq!(outcome.current.status, TaskStatus::Done as i32);
    assert!(outcome.rewards.items.is_empty());
    assert!(outcome.level_up_data.is_none());
    assert_eq!(
        outcome
            .changed
            .iter()
            .map(|task| task.id)
            .collect::<Vec<_>>(),
        [1_500_001_001]
    );
    assert_eq!(outcome.changed[0].status, TaskStatus::Picked as i32);
    assert_eq!(outcome.changed[0].picked_at, 123);
}

#[test]
fn task_rewards_use_their_configured_game_types() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .tasks
        .extend(
            [1_501_003_060, 1_501_003_100, 1_501_004_040].map(|id| DcNetDataTaskStatus {
                id,
                status: TaskStatus::Picked as i32,
                ..Default::default()
            }),
        );

    let partner = player
        .complete_task(1_501_003_060, 0, &tables, 123)
        .unwrap()
        .rewards;
    assert_eq!(
        partner.partner_rewards[0].partner.unwrap().partner_id,
        100_400_026
    );
    assert_eq!(player.partners.len(), 1);

    let equip = player
        .complete_task(1_501_003_100, 0, &tables, 124)
        .unwrap()
        .rewards;
    assert_eq!(equip.equips[0].equip_id, 100_500_114);
    assert_eq!(equip.equips[0].pos, 1);
    assert_eq!(player.equips.len(), 1);

    let collection = player
        .complete_task(1_501_004_040, 0, &tables, 125)
        .unwrap()
        .rewards;
    assert_eq!(collection.collection_rewards[0].cid, 100_700_016);
    assert_eq!(player.collections.len(), 1);
}

#[test]
fn task_rewards_apply_table_driven_player_exp() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_500_001_240,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcome = player
        .complete_task(1_500_001_240, 0, &tables, 123)
        .unwrap();
    let level_up = outcome.level_up_data.unwrap();
    assert_eq!((level_up.attr_lv, level_up.add_exp), (3, 400));
    assert_eq!(level_up.items[0].amount, 80);
    assert_eq!(player.level, 3);
    assert_eq!(outcome.rewards.items[0], level_up.items[0]);
    assert!(
        outcome
            .archive_updates
            .iter()
            .any(|archive| archive.id == 3001)
    );
}

#[test]
fn task_level_rewards_report_newly_completed_achievements() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 19;
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_007_160,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });

    let outcome = player
        .complete_task(1_501_007_160, 0, &tables, 123)
        .unwrap();

    assert!(player.level >= 20);
    assert!(outcome.achievement_updates.iter().any(|achievement| {
        achievement.id == 100_302 && achievement.progress == 20 && achievement.total == 20
    }));
}

#[test]
fn story_tasks_do_not_advance_dungeon_daily_objectives() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.daily_tasks = vec![DcNetDataTaskCycle {
        id: 10,
        progress: 0,
        total: 3,
        ..Default::default()
    }];

    player
        .complete_task(1_500_000_000, 0, &tables, 123)
        .unwrap();

    assert_eq!(player.daily_tasks[0].progress, 0);
}
