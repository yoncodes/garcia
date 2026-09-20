use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn rift_run_advances_persists_and_settles_only_on_the_final_stage() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let event = tables.rifts.get(8).unwrap();
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;
    let buffs = HashMap::from([(6001, 1), (6002, 3)]);

    let ticket_id = tables.cultivation_constants.ticket_item_id;
    let tickets_before = player
        .items
        .iter()
        .find(|item| item.item_id == ticket_id)
        .unwrap()
        .amount;
    let (port_id, remains) = player
        .start_rift(event, &buffs, vec![role_id], &tables)
        .unwrap();
    assert_eq!(port_id, 180_706_001);
    assert_eq!(remains[0].item_id, ticket_id);
    assert_eq!(remains[0].amount, tickets_before - 1);
    assert_eq!(player.rift.run_buff_score, 4);
    assert_eq!(
        player.settle_rift(event, 10, &tables),
        Err(RiftError::NotFinalStage(180_706_001))
    );

    let mut expected_score = 0;
    for expected_step in 2..=10 {
        let stage = tables.rift_ports.get(player.rift.current_port_id).unwrap();
        expected_score += rift_stage_score(stage, 10);
        let (duration, port_id) = player.report_rift_stage(event, 10, &tables).unwrap();
        assert_eq!(duration, (expected_step - 1) * 10);
        assert_eq!(port_id, 180_706_000 + expected_step);
    }
    let final_stage = tables.rift_ports.get(player.rift.current_port_id).unwrap();
    expected_score += rift_stage_score(final_stage, 20);
    assert_eq!(
        player.settle_rift(event, 20, &tables).unwrap(),
        (110, expected_score, 180_706_010, 4)
    );
    assert_eq!(
        (
            player.rift.best_stage,
            player.rift.completion_count,
            player.rift.best_score
        ),
        (10, 1, expected_score)
    );
    player.claim_rift_task(event, 6001, &tables, 123).unwrap();
    assert_eq!(
        player.claim_rift_task(event, 6001, &tables, 124),
        Err(RiftError::TaskAlreadyClaimed(6001))
    );
    assert_eq!(
        player.claim_rift_task(event, 6017, &tables, 125),
        Err(RiftError::TaskNotComplete(6017))
    );
    assert_eq!(
        player.report_rift_stage(event, 1, &tables),
        Err(RiftError::NoActiveRun)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.rift, player.rift);
}
