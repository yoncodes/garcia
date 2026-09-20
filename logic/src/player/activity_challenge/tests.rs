use super::*;

#[test]
fn activity_challenges_follow_unlock_chain_claim_rewards_and_persist() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let event = tables.activities.get(7).unwrap();
    let now = (common::time::table_time_utc(&event.open_time, config.server.zone_offset).unwrap()
        + 60) as i32;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let unlock_task = 1_501_001_060;
    if let Some(task) = player.tasks.iter_mut().find(|task| task.id == unlock_task) {
        task.status = TaskStatus::Done as i32;
    } else {
        player.tasks.push(DcNetDataTaskStatus {
            id: unlock_task,
            status: TaskStatus::Done as i32,
            ..Default::default()
        });
    }
    let mut rows = tables
        .activity_challenges
        .rows
        .iter()
        .filter(|row| row.activity_id == 7)
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.times);

    let (list, claimed_points) = player.activity_challenges(7, &tables).unwrap();
    assert_eq!(list.len(), 7);
    assert!(claimed_points.is_empty());
    assert!(
        list.iter()
            .find(|entry| entry.id == rows[0].id)
            .unwrap()
            .enabled
    );
    assert!(
        !list
            .iter()
            .find(|entry| entry.id == rows[1].id)
            .unwrap()
            .enabled
    );

    player
        .set_challenge_result(&rows[0].challenge_id, 3)
        .unwrap();
    let first = player
        .claim_activity_challenge(rows[0].id, &tables, now, config.server.zone_offset)
        .unwrap();
    assert!(!first.items.is_empty());
    assert!(
        player
            .activity_challenges(7, &tables)
            .unwrap()
            .0
            .iter()
            .find(|entry| entry.id == rows[1].id)
            .unwrap()
            .enabled
    );

    player
        .set_challenge_result(&rows[1].challenge_id, 3)
        .unwrap();
    player
        .claim_activity_challenge(rows[1].id, &tables, now, config.server.zone_offset)
        .unwrap();
    let point = tables
        .activity_challenge_milestones
        .rows
        .iter()
        .find(|point| point.activity_id == 7 && point.challenge_point == 2)
        .unwrap();
    let reward = player
        .claim_activity_challenge_point(point.id, &tables, now, config.server.zone_offset)
        .unwrap();
    assert!(!reward.items.is_empty());
    assert_eq!(
        player.claim_activity_challenge_point(point.id, &tables, now, config.server.zone_offset,),
        Err(ActivityChallengeError::PointAlreadyClaimed(point.id))
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored.activity_challenge_point_claims,
        player.activity_challenge_point_claims
    );
    assert_eq!(restored.challenges, player.challenges);
}
