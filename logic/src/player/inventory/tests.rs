use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn item_grant_validates_updates_and_survives_reload() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let item_id = tables.cultivation_constants.diamond_item_id;
    let before = player
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .map_or(0, |item| item.amount);

    let reward = player.grant_reward(item_id, 500, &tables, 0).unwrap();
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (item_id, before + 500)
    );
    assert_eq!(
        player.grant_reward(item_id, 0, &tables, 0),
        Err(InventoryError::InvalidAmount(0))
    );
    assert_eq!(
        player.grant_reward(-1, 1, &tables, 0),
        Err(InventoryError::UnknownReward(-1))
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .unwrap()
            .amount,
        before + 500
    );
}

#[test]
fn item_use_and_selectable_packages_apply_extracted_rewards() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let consumable = player.add_item(100_300_001, 2, &tables).unwrap();
    let used = player
        .use_item(
            DcNetDataUseItem {
                user_item_id: consumable.user_item_id,
                item_id: consumable.item_id,
                amount: 2,
                ..Default::default()
            },
            &tables,
            0,
        )
        .unwrap();
    assert_eq!(used.remain.amount, 0);
    assert_eq!(used.rewards.items[0].item_id, 100_100_005);
    assert_eq!(used.rewards.items[0].amount, 2_000);

    player.add_item(100_302_001, 2, &tables).unwrap();
    let selected = player
        .select_package_reward(100_302_001, &[(100_301_301, 2)], &tables, 0)
        .unwrap();
    assert_eq!(selected.remain.amount, 0);
    assert_eq!(
        (
            selected.rewards.items[0].item_id,
            selected.rewards.items[0].amount
        ),
        (100_301_301, 2)
    );

    player.add_item(100_302_001, 1, &tables).unwrap();
    let before = player.items.clone();
    assert_eq!(
        player.select_package_reward(100_302_001, &[(100_100_003, 1)], &tables, 0),
        Err(InventoryError::InvalidSelection(100_100_003))
    );
    assert_eq!(player.items, before);
}

#[test]
fn sacred_white_stones_convert_to_table_defined_emberite() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let stone = player.add_item(100_100_001, 3, &tables).unwrap();
    let emberite_before = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_002)
        .map_or(0, |item| item.amount);

    let used = player
        .use_item(
            DcNetDataUseItem {
                user_item_id: stone.user_item_id,
                item_id: stone.item_id,
                amount: 2,
                ..Default::default()
            },
            &tables,
            0,
        )
        .unwrap();

    assert_eq!(used.remain.amount, 1);
    assert_eq!(used.rewards.items.len(), 1);
    assert_eq!(used.rewards.items[0].item_id, 100_100_002);
    assert_eq!(used.rewards.items[0].amount, emberite_before + 20);
}

#[test]
fn random_relic_boxes_grant_one_table_relic_per_box() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let box_id = 100_305_003;
    let item = player.add_item(box_id, 2, &tables).unwrap();
    let possible = tables
        .rewards
        .get(
            tables
                .items
                .get(box_id)
                .unwrap()
                .effect
                .as_ref()
                .unwrap()
                .key,
        )
        .unwrap()
        .reward
        .iter()
        .map(|reward| reward.key)
        .collect::<Vec<_>>();

    let used = player
        .use_item(
            DcNetDataUseItem {
                user_item_id: item.user_item_id,
                item_id: box_id,
                amount: 2,
                ..Default::default()
            },
            &tables,
            123,
        )
        .unwrap();

    assert_eq!(used.remain.amount, 0);
    assert_eq!(used.rewards.equips.len(), 2);
    assert!(
        used.rewards
            .equips
            .iter()
            .all(|relic| possible.contains(&relic.equip_id))
    );
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.equips.len(), player.equips.len());
}

#[test]
fn ordinary_type_102_packages_still_grant_every_table_reward() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let package_id = 100_302_100;
    let item = player.add_item(package_id, 1, &tables).unwrap();

    let used = player
        .use_item(
            DcNetDataUseItem {
                user_item_id: item.user_item_id,
                item_id: package_id,
                amount: 1,
                ..Default::default()
            },
            &tables,
            123,
        )
        .unwrap();

    assert_eq!(used.remain.amount, 0);
    assert_eq!(used.rewards.items.len(), 2);
    assert!(
        used.rewards
            .items
            .iter()
            .any(|item| item.item_id == 100_100_025)
    );
    assert!(
        used.rewards
            .items
            .iter()
            .any(|item| item.item_id == 100_100_003)
    );
}

#[test]
fn item_use_caps_the_configured_heat_item() {
    let (mut tables, config) = fixture();
    let replacement = tables.cultivation_constants.diamond_item_id;
    tables.cultivation_constants.heat_item_id = replacement;
    tables
        .items
        .rows
        .iter_mut()
        .find(|item| item.id == 100_300_000)
        .unwrap()
        .effect
        .as_mut()
        .unwrap()
        .key = replacement;

    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .add_item(
            replacement,
            tables.cultivation_constants.heat_limit - 1,
            &tables,
        )
        .unwrap();
    let consumable = player.add_item(100_300_000, 1, &tables).unwrap();

    let used = player
        .use_item(
            DcNetDataUseItem {
                user_item_id: consumable.user_item_id,
                item_id: consumable.item_id,
                amount: 1,
                ..Default::default()
            },
            &tables,
            0,
        )
        .unwrap();

    assert_eq!(used.rewards.items[0].item_id, replacement);
    assert_eq!(
        used.rewards.items[0].amount,
        tables.cultivation_constants.heat_limit
    );
}
