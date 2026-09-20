use super::*;

#[test]
fn unlocking_teleport_gates_is_persistent_and_idempotent() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let updates = player.unlock_all_teleport_gates(&tables);
    assert_eq!(updates.len(), tables.teleportation_anchors.rows.len());
    assert!(
        updates
            .iter()
            .all(|gate| gate.status == 1 && gate.interactive)
    );
    assert!(player.unlock_all_teleport_gates(&tables).is_empty());
}

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn custom_formations_use_the_extracted_fixed_and_dynamic_limits() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    assert_eq!(
        player
            .formations
            .iter()
            .map(|formation| formation.formation_id)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );

    for expected in 6..11 {
        assert_eq!(
            player.add_formation(&tables).unwrap().formation_id,
            expected
        );
    }
    assert_eq!(
        player.add_formation(&tables),
        Err(FormationError::LimitReached)
    );
    assert_eq!(
        player.delete_formation(0, &tables),
        Err(FormationError::Fixed(0))
    );
    assert!(player.select_formation(6).unwrap());
    assert_eq!(
        player.delete_formation(6, &tables),
        Err(FormationError::Current(6))
    );
    assert!(player.select_formation(1).unwrap());
    player.delete_formation(6, &tables).unwrap();

    let mut oversized = player.formations[0].clone();
    oversized.poss = vec![
        DcNetDataFormationPos::default();
        tables.cultivation_constants.formation_member_limit as usize + 1
    ];
    assert!(matches!(
        player.set_formation(oversized, &tables),
        Err(FormationError::TooManyMembers { .. })
    ));
    assert_eq!(
        player.select_formation(99),
        Err(FormationError::Unknown(99))
    );

    let mut restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert!(
        !restored
            .formations
            .iter()
            .any(|formation| formation.formation_id == 6)
    );
    assert_eq!(
        restored.add_formation(&tables),
        Ok(DcNetDataFormation {
            formation_id: 6,
            ..Default::default()
        })
    );
}

#[test]
fn playing_port_updates_once_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert!(player.set_playing_port(140_301_056, "checkpoint".into()));
    assert!(!player.set_playing_port(140_301_056, "checkpoint".into()));

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.playing_port, player.playing_port);
}

#[test]
fn first_city_guide_interaction_unlocks_its_captured_archive() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let guide = "guide_chruch_steal";

    assert!(player.record_city_guide(guide.to_owned(), &tables).unwrap());
    player.set_interact(guide, 1).unwrap();
    assert!(
        player
            .interact_objs
            .iter()
            .any(|object| { object.object_id == guide && object.status == 1 && object.count == 1 })
    );
    assert!(player.city_guides.iter().any(|id| id == guide));
    assert!(tables.archives.get(5512).is_some());
    assert_eq!(
        player.newly_unlocked_interaction_archives(guide, &tables, 1_789_761_670),
        [DcNetDataArchiveInfo {
            id: 5512,
            in_time: 1_789_761_670,
        }]
    );

    player.set_interact(guide, 1).unwrap();
    assert!(
        player
            .newly_unlocked_interaction_archives(guide, &tables, 1_789_761_671)
            .is_empty()
    );
}

#[test]
fn tutorial_groups_can_be_completed_and_reset() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert_eq!(player.complete_all_tutorials(&tables), 11);
    assert_eq!(player.complete_all_tutorials(&tables), 0);
    let mut restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.user_guides.len(), 11);
    assert_eq!(restored.reset_tutorials(&tables), 11);
    assert!(restored.user_guides.is_empty());
}

#[test]
fn reward_interactions_grant_the_table_bundle_once() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let object_id = "nh_ashsnow_box_slate_1";

    let outcome = player
        .complete_interaction(object_id, 1, &tables, 1_789_761_670)
        .unwrap();
    let reward = outcome.reward.unwrap();
    assert_eq!(
        reward
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_307_904, 1)]
    );
    assert_eq!(
        (outcome.object.object_id.as_str(), outcome.object.count),
        (object_id, 1)
    );
    let duplicate = player
        .complete_interaction(object_id, 1, &tables, 1_789_761_671)
        .unwrap();
    assert!(duplicate.reward.is_none());
    assert_eq!(duplicate.object.count, 1);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_307_904)
            .map(|item| item.amount),
        Some(1)
    );
}

#[test]
fn city_rush_grants_only_new_table_reward_tiers_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let id = "city_rush_test";
    let item_amount = |player: &Player, item_id| {
        player
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .map_or(0, |item| item.amount)
    };
    let first_before = item_amount(&player, 100_301_113);
    let second_before = item_amount(&player, 100_300_005);

    let (challenge, reward) = player
        .complete_city_challenge(id, 1, &tables, 1_789_761_670)
        .unwrap();
    assert_eq!((challenge.stars, challenge.finished), (1, true));
    assert_eq!(
        reward
            .unwrap()
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [
            (100_301_113, first_before + 2),
            (100_300_005, second_before + 2)
        ]
    );

    let (_, duplicate) = player
        .complete_city_challenge(id, 1, &tables, 1_789_761_671)
        .unwrap();
    assert!(duplicate.unwrap().items.is_empty());

    let (challenge, reward) = player
        .complete_city_challenge(id, 3, &tables, 1_789_761_672)
        .unwrap();
    assert_eq!(challenge.stars, 3);
    assert_eq!(reward.unwrap().items.len(), 2);
    assert_eq!(item_amount(&player, 100_301_113), first_before + 6);
    assert_eq!(item_amount(&player, 100_300_005), second_before + 6);

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    let restored_challenge = restored
        .challenges
        .iter()
        .find(|challenge| challenge.id == id)
        .unwrap();
    assert_eq!(
        (restored_challenge.stars, restored_challenge.finished),
        (3, true)
    );
}

#[test]
fn invalid_city_rush_stars_do_not_mutate_state() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let before = player.to_record();

    assert_eq!(
        player.complete_city_challenge("city_rush_test", 4, &tables, 1_789_761_670),
        Err(CityChallengeError::InvalidStars {
            id: "city_rush_test".into(),
            stars: 4,
        })
    );
    assert_eq!(player.to_record(), before);
}
