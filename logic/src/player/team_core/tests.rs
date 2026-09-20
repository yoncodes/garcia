use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    (tables, config)
}

#[test]
fn team_core_synthesis_spends_tables_locks_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.items.extend([
        DcNetDataItem {
            user_item_id: make_entity_id(player.uid, 100_300_031),
            item_id: 100_300_031,
            amount: 4,
            ..Default::default()
        },
        DcNetDataItem {
            user_item_id: make_entity_id(player.uid, 100_300_030),
            item_id: 100_300_030,
            amount: 20,
            ..Default::default()
        },
    ]);

    let (created, remains) = player.compose_team_cores(1, 2, &tables).unwrap();
    assert_eq!(created.len(), 2);
    assert!(
        created
            .iter()
            .all(|core| tables.team_cores.get(core.core_id).is_some())
    );
    assert_eq!(
        remains
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_300_030, 0), (100_300_031, 0)]
    );

    let locked = player.lock_team_core(created[0].id, true).unwrap();
    assert!(locked.locked);
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.team_cores, player.team_cores);
}

#[test]
fn team_core_synthesis_rejects_invalid_requests_atomically() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.items.extend([
        DcNetDataItem {
            user_item_id: make_entity_id(player.uid, 100_300_031),
            item_id: 100_300_031,
            amount: 1,
            ..Default::default()
        },
        DcNetDataItem {
            user_item_id: make_entity_id(player.uid, 100_300_030),
            item_id: 100_300_030,
            amount: 9,
            ..Default::default()
        },
    ]);
    let before = player.items.clone();

    assert_eq!(
        player.compose_team_cores(1, 1, &tables),
        Err(TeamCoreError::InsufficientCost {
            item_id: 100_300_030,
            needed: 10,
            available: 9,
        })
    );
    assert_eq!(player.items, before);
    assert!(player.team_cores.is_empty());
    assert_eq!(
        player.lock_team_core(99, true),
        Err(TeamCoreError::UnknownCore(99))
    );
}
