use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig, Player) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let player = Player::new(42, &tables, &config.account_defaults);
    (tables, config, player)
}

#[test]
fn suit_rewards_use_table_thresholds_and_persist() {
    let (mut tables, config, _) = fixture();
    tables.cultivation_constants.diamond_item_id = 100_100_003;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.collections.push(DcNetDataCollection {
        cid: 100_700_010,
        in_time: 1,
        reward: false,
    });

    let (reward, steps) = player
        .claim_collection_suit_reward(10_070_004, 1, &tables, 2)
        .unwrap();
    assert_eq!(steps, [1]);
    assert_eq!(reward.items[0].item_id, 100_100_003);
    assert!(matches!(
        player.claim_collection_suit_reward(10_070_004, 1, &tables, 2),
        Err(CollectionError::SuitRewardClaimed { .. })
    ));
    assert!(matches!(
        player.claim_collection_suit_reward(10_070_004, 2, &tables, 2),
        Err(CollectionError::SuitIncomplete { required: 3, .. })
    ));

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored.collection_suit_rewards,
        player.collection_suit_rewards
    );
}

#[test]
fn display_placement_moves_clears_and_persists() {
    let (tables, config, mut player) = fixture();
    player.collections.extend([
        DcNetDataCollection {
            cid: 100_700_010,
            ..Default::default()
        },
        DcNetDataCollection {
            cid: 100_700_027,
            ..Default::default()
        },
    ]);

    assert_eq!(
        player.place_collection(100_700_010, 1, &tables).unwrap(),
        [DcNetDataCollectionPlaceInfo {
            cid: 100_700_010,
            pid: 1,
        }]
    );
    let moved = player.place_collection(100_700_010, 2, &tables).unwrap();
    assert_eq!(moved[0], DcNetDataCollectionPlaceInfo { cid: 0, pid: 1 });
    assert_eq!(moved[1].pid, 2);
    player.place_collection(100_700_027, 1, &tables).unwrap();
    player.place_collection(0, 2, &tables).unwrap();
    assert_eq!(
        player.collection_places,
        [DcNetDataCollectionPlaceInfo {
            cid: 100_700_027,
            pid: 1,
        }]
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.collection_places, player.collection_places);
}

#[test]
fn collection_mutations_reject_unowned_and_unconfigured_rewards() {
    let (tables, _, mut player) = fixture();
    assert_eq!(
        player.place_collection(100_700_010, 1, &tables),
        Err(CollectionError::CollectionNotOwned(100_700_010))
    );
    player.collections.push(DcNetDataCollection {
        cid: 100_700_010,
        ..Default::default()
    });
    assert_eq!(
        player.claim_collection_reward(100_700_010, &tables, 2),
        Err(CollectionError::MissingReward(100_700_010))
    );
}
