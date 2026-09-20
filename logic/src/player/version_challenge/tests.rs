use super::*;

#[test]
fn version_challenges_unlock_accumulate_stars_claim_and_persist() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 100;
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_001_060,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    let first = tables
        .version_challenges
        .rows
        .iter()
        .find(|challenge| challenge.activity_id == 1003 && challenge.pre_dungeon_id == 0)
        .unwrap();
    let now = (common::time::table_time_utc(&first.open_time, config.server.zone_offset).unwrap()
        + 60) as i32;

    let stages = player.version_challenge_stages(&tables, now, config.server.zone_offset);
    assert_eq!(stages.len(), 18);
    assert_eq!(
        stages
            .iter()
            .find(|stage| stage.id == first.id)
            .unwrap()
            .countdown,
        0
    );
    player
        .start_version_challenge(first.id, &tables, now, config.server.zone_offset)
        .unwrap();
    let partial = player
        .settle_version_challenge(
            first.id,
            &HashMap::from([(1, 120), (2, 0), (3, 0)]),
            &tables,
            now,
            config.server.zone_offset,
        )
        .unwrap();
    assert_eq!(
        (partial.star1, partial.star2, partial.star3),
        (true, true, false)
    );
    let complete = player
        .settle_version_challenge(
            first.id,
            &HashMap::from([(1, 80), (2, 0), (3, 0)]),
            &tables,
            now,
            config.server.zone_offset,
        )
        .unwrap();
    assert_eq!(
        (complete.star1, complete.star2, complete.star3),
        (true, true, true)
    );

    let (reward, reward_ids) = player
        .claim_version_challenge_rewards(&tables, now, config.server.zone_offset)
        .unwrap();
    assert_eq!(reward_ids.len(), 3);
    assert!(!reward.items.is_empty());
    assert!(
        player
            .claim_version_challenge_rewards(&tables, now, config.server.zone_offset)
            .unwrap()
            .1
            .is_empty()
    );

    let second = tables
        .version_challenges
        .rows
        .iter()
        .find(|challenge| challenge.pre_dungeon_id == first.id)
        .unwrap();
    let second_open =
        common::time::table_time_utc(&second.open_time, config.server.zone_offset).unwrap() as i32;
    player
        .start_version_challenge(second.id, &tables, second_open, config.server.zone_offset)
        .unwrap();

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.version_challenges, player.version_challenges);
}

#[test]
fn version_challenge_rejects_missing_settlement_metrics() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 100;
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_001_060,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    let first = tables
        .version_challenges
        .rows
        .iter()
        .find(|challenge| challenge.activity_id == 1003 && challenge.pre_dungeon_id == 0)
        .unwrap();
    let now =
        common::time::table_time_utc(&first.open_time, config.server.zone_offset).unwrap() as i32;

    assert_eq!(
        player.settle_version_challenge(
            first.id,
            &HashMap::from([(1, 100)]),
            &tables,
            now,
            config.server.zone_offset,
        ),
        Err(VersionChallengeError::InvalidResult(first.id))
    );
}
