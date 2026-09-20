use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn synthesis_and_conversion_use_extracted_formulas_atomically() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 58;
    player.add_item(100_301_002, 6, &tables).unwrap();

    let synthesis = player.synthesize_item(1, 2, &tables, 0).unwrap();
    assert_eq!((synthesis.id, synthesis.amount), (1, 2));
    assert_eq!(
        (synthesis.remains[0].item_id, synthesis.remains[0].amount),
        (100_301_002, 0)
    );
    assert_eq!(
        (synthesis.target.item_id, synthesis.target.amount),
        (100_301_003, 2)
    );

    player.add_item(100_301_006, 2, &tables).unwrap();
    player.add_item(100_301_009, 2, &tables).unwrap();
    let conversion = player
        .convert_item(1, 2, &[100_301_006, 100_301_009], &tables, 0)
        .unwrap();
    assert_eq!(
        conversion
            .remains
            .iter()
            .map(|item| item.amount)
            .collect::<Vec<_>>(),
        [0, 0]
    );
    assert_eq!(conversion.target.amount, 4);

    let before = player.items.clone();
    assert_eq!(
        player.synthesize_item(1, 1, &tables, 0),
        Err(CraftError::InsufficientMaterial {
            item_id: 100_301_002,
            needed: 3,
            available: 0,
        })
    );
    assert_eq!(player.items, before);
}
