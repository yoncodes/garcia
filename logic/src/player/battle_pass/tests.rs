use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn battle_pass_claims_match_captured_new_account_flow_and_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_522_872;
    player.created_at = now - 86_400;
    let zone_offset = config.server.zone_offset;
    let coin_item_id = tables.cultivation_constants.coin_item_id;
    if let Some(gold) = player
        .items
        .iter_mut()
        .find(|item| item.item_id == coin_item_id)
    {
        gold.amount = 60_100;
    } else {
        player.add_item(coin_item_id, 60_100, &tables).unwrap();
    }

    let (info, changed) = player.battle_pass_info(&tables, now, zone_offset).unwrap();
    assert!(changed);
    assert_eq!(
        (info.id, info.exp_week_limit, info.exp_per_level),
        (1003, 9000, 1000)
    );
    assert_eq!((info.battle_pass.level, info.battle_pass.exp), (1, 0));

    let (tasks, _, _, _) = player.battle_pass_tasks(&tables, now, zone_offset).unwrap();
    assert_eq!(tasks.len(), 13);
    let task = |id| tasks.iter().find(|task| task.id == id).unwrap();
    assert_eq!((task(10_031_010).progress, task(10_031_010).total), (2, 1));
    assert_eq!(
        (task(10_031_040).progress, task(10_031_040).total),
        (60_100, 50_000)
    );

    let (reward, battle_pass) = player
        .claim_battle_pass_rewards(0, &tables, now, zone_offset)
        .unwrap();
    let reward_updates = player.battle_pass_reward_updates(&reward, &tables, now, zone_offset);
    assert_eq!(
        reward
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(coin_item_id, 180_100)]
    );
    assert_eq!(battle_pass.taken.get(&1), Some(&1));
    assert_eq!(reward_updates.len(), 1);
    assert_eq!(reward_updates[0].id, 10_031_040);
    assert!(reward_updates[0].progress >= reward_updates[0].total);

    let (claimed, battle_pass) = player
        .claim_battle_pass_tasks(0, &tables, now, zone_offset)
        .unwrap();
    assert_eq!(
        claimed.iter().map(|task| task.id).collect::<Vec<_>>(),
        [10_031_010, 10_031_040]
    );
    assert!(
        claimed
            .iter()
            .all(|task| task.status == TaskStatus::Done as i32)
    );
    assert_eq!(
        (battle_pass.level, battle_pass.exp, battle_pass.exp_week),
        (1, 525, 525)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.battle_pass, player.battle_pass);
}

#[test]
fn battle_pass_level_cost_uses_the_configured_diamond_item() {
    let (mut tables, config) = fixture();
    let coin_item_id = tables.cultivation_constants.coin_item_id;
    tables.cultivation_constants.diamond_item_id = coin_item_id;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_522_872;
    let price = tables.cultivation_constants.battle_pass_level_price;
    player.add_item(coin_item_id, price, &tables).unwrap();

    let (remain, battle_pass) = player
        .level_up_battle_pass(1, &tables, now, config.server.zone_offset)
        .unwrap();

    assert_eq!(remain.item_id, coin_item_id);
    assert_eq!(battle_pass.level, 2);
}

#[test]
fn battle_pass_queries_are_empty_after_the_last_configured_season() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let end = common::time::table_time_utc(
        &tables.battle_passes.rows.last().unwrap().end_time,
        config.server.zone_offset,
    )
    .unwrap() as i32;

    assert!(
        player
            .battle_pass_info(&tables, end + 1, config.server.zone_offset)
            .is_none()
    );
    assert!(
        player
            .battle_pass_tasks(&tables, end + 1, config.server.zone_offset)
            .is_none()
    );
}
