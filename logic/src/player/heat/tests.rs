use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    (tables, config)
}

#[test]
fn heat_regenerates_from_table_interval_and_level_cap() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let heat = player
        .items
        .iter_mut()
        .find(|item| item.item_id == tables.cultivation_constants.heat_item_id)
        .unwrap();
    heat.amount = 200;
    player.heat_updated_at = 100;

    let before_tick = player.refresh_heat(&tables, 459).unwrap();
    assert_eq!((before_tick.item.amount, before_tick.cd_time), (200, 1));

    let tick = player.refresh_heat(&tables, 460).unwrap();
    assert_eq!((tick.item.amount, tick.cd_time), (201, 360));
    assert_eq!(player.heat_updated_at, 460);

    let full = player.refresh_heat(&tables, 460 + 39 * 360).unwrap();
    assert_eq!((full.item.amount, full.cd_time), (240, 0));
}

#[test]
fn heat_regeneration_timestamp_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.heat_updated_at = player.created_at + 123;

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.heat_updated_at, player.heat_updated_at);
}
#[test]
fn heat_exchange_uses_table_price_tiers_daily_limit_and_persistence() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.add_item(100_100_002, 200, &tables).unwrap();
    let now = 1_700_000_000;

    let first = player.exchange_heat(1, &tables, now).unwrap();
    assert_eq!(
        (first.cost_remain.amount, first.heat_remain.amount),
        (140, 300)
    );
    let second = player.exchange_heat(1, &tables, now).unwrap();
    assert_eq!(
        (second.cost_remain.amount, second.heat_remain.amount),
        (80, 360)
    );
    assert_eq!(second.exchange_count, 2);

    let before = player.to_record();
    assert_eq!(
        player.exchange_heat(1, &tables, now),
        Err(HeatExchangeError::InsufficientCost {
            item_id: 100_100_002,
            needed: 120,
            available: 80,
        })
    );
    assert_eq!(player.to_record(), before);
    let restored = Player::from_record(before, &tables, &config.account_defaults);
    assert_eq!(restored.heat_exchange_count(now), 2);
    assert_eq!(restored.heat_exchange_count(now + 86_400), 0);
}

#[test]
fn diamond_exchange_uses_the_extracted_stamp_rate() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .add_item(tables.cultivation_constants.diamond_item_id, 500, &tables)
        .unwrap();

    let changed = player.exchange_diamonds_for_stamps(3, &tables).unwrap();
    assert_eq!(
        changed
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [
            (tables.cultivation_constants.diamond_item_id, 20),
            (tables.cultivation_constants.stamp_item_id, 3),
        ]
    );
    assert_eq!(
        player.exchange_diamonds_for_stamps(1, &tables),
        Err(CurrencyExchangeError::InsufficientDiamonds {
            needed: 160,
            available: 20,
        })
    );
}
