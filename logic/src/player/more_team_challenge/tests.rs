use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn more_team_challenges_unlock_keep_best_time_claim_and_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 100;
    let first = tables
        .more_team_challenges
        .rows
        .iter()
        .find(|challenge| challenge.group_id == 10_010 && challenge.pre_dungeon_id == 0)
        .unwrap();

    let (groups, changed) =
        player.more_team_challenge_groups(&tables, 0, config.server.zone_offset);
    assert!(!changed);
    assert_eq!(
        groups
            .iter()
            .find(|group| group.gid == first.group_id)
            .unwrap()
            .countdown,
        -1
    );
    player
        .start_more_team_challenge(first.id, &tables, 0, config.server.zone_offset)
        .unwrap();
    let partial = player
        .settle_more_team_challenge(first.id, 120, &tables, 0, config.server.zone_offset)
        .unwrap();
    assert_eq!(
        (
            partial.cost_time,
            partial.star1,
            partial.star2,
            partial.star3
        ),
        (120, true, true, false)
    );
    let complete = player
        .settle_more_team_challenge(first.id, 80, &tables, 0, config.server.zone_offset)
        .unwrap();
    assert_eq!(
        (
            complete.cost_time,
            complete.star1,
            complete.star2,
            complete.star3,
        ),
        (80, true, true, true)
    );

    let (_, reward_ids, changed) =
        player.claim_more_team_challenge_rewards(&tables, 0, config.server.zone_offset);
    assert!(changed);
    assert_eq!(reward_ids.len(), 3);
    assert!(
        player
            .claim_more_team_challenge_rewards(&tables, 0, config.server.zone_offset)
            .1
            .is_empty()
    );

    let second = tables
        .more_team_challenges
        .rows
        .iter()
        .find(|challenge| challenge.pre_dungeon_id == first.id)
        .unwrap();
    player
        .start_more_team_challenge(second.id, &tables, 0, config.server.zone_offset)
        .unwrap();

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.more_team_challenges, player.more_team_challenges);
}

#[test]
fn more_team_challenge_rejects_non_positive_completion_time() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 100;

    assert_eq!(
        player.settle_more_team_challenge(1, 0, &tables, 0, config.server.zone_offset),
        Err(MoreTeamChallengeError::InvalidTime(1))
    );
}
