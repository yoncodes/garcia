use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn favor_touch_and_gift_use_tables_reset_daily_and_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let character_id = tables.maids.get(110_100_013).unwrap().character_id;
    let now = 1_787_522_872;

    let (touches, favor) = player
        .touch_favor(character_id, &tables, now, config.server.zone_offset)
        .unwrap();
    assert_eq!((touches, favor.lv, favor.exp), (1, 1, 20));

    player.add_item(100_306_003, 1, &tables).unwrap();
    let (favor, remains) = player
        .gift_favor(
            character_id,
            &[DcNetDataUseItem {
                item_id: 100_306_003,
                amount: 1,
                ..Default::default()
            }],
            &tables,
        )
        .unwrap();
    assert_eq!((favor.lv, favor.exp), (2, 20));
    assert_eq!((remains[0].item_id, remains[0].amount), (100_306_003, 0));

    let next_day = now + 86_400;
    assert_eq!(
        player.favor_touches(&tables, next_day, config.server.zone_offset),
        0
    );
    let (touches, favor) = player
        .touch_favor(character_id, &tables, next_day, config.server.zone_offset)
        .unwrap();
    assert_eq!((touches, favor.lv, favor.exp), (1, 2, 40));

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.favors, player.favors);
    assert_eq!(restored.favor_touches, 1);
    assert_eq!(restored.favor_day, player.favor_day);
}

#[test]
fn favor_rejects_unknown_characters_and_non_favor_items_atomically() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    assert_eq!(
        player
            .touch_favor(1, &tables, 1_787_522_872, config.server.zone_offset)
            .unwrap_err(),
        FavorError::UnknownCharacter(1)
    );

    let before = player.items.clone();
    assert_eq!(
        player
            .gift_favor(
                110_300_013,
                &[DcNetDataUseItem {
                    item_id: 100_100_008,
                    amount: 1,
                    ..Default::default()
                }],
                &tables,
            )
            .unwrap_err(),
        FavorError::InvalidMaterial(100_100_008)
    );
    assert_eq!(player.items, before);
    assert!(player.favors.is_empty());
}

#[test]
fn favor_gift_uses_the_table_effect_value_without_assuming_its_key() {
    let (mut tables, config) = fixture();
    let gift = tables
        .items
        .rows
        .iter_mut()
        .find(|item| item.id == 100_306_003)
        .unwrap();
    let effect = gift.effect.as_mut().unwrap();
    effect.key = 987_654_321;
    effect.value = 37;
    let gift_id = gift.id;
    let expected_exp = gift.effect.as_ref().unwrap().value;

    let mut player = Player::new(42, &tables, &config.account_defaults);
    let character_id = tables.maids.get(110_100_013).unwrap().character_id;
    player.add_item(gift_id, 1, &tables).unwrap();

    let (favor, remains) = player
        .gift_favor(
            character_id,
            &[DcNetDataUseItem {
                item_id: gift_id,
                amount: 1,
                ..Default::default()
            }],
            &tables,
        )
        .unwrap();

    assert_eq!(favor.exp, expected_exp);
    assert_eq!(remains[0].amount, 0);
}
