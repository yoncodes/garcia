use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn equipment_and_partner_disassembly_uses_extracted_refund_formulas() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let equip_id = tables
        .equipment
        .rows
        .iter()
        .find(|row| row.quality == 4)
        .unwrap()
        .id;
    let partner_id = tables
        .partners
        .rows
        .iter()
        .find(|row| row.quality == 4)
        .unwrap()
        .id;
    player.equips.push(DcNetDataEquip {
        equip_id,
        user_equip_id: 7001,
        level: 2,
        exp: 100,
        quality: 4,
        ..Default::default()
    });
    player.partners.push(DcNetDataPartner {
        id: 8001,
        partner_id,
        lv: 3,
        exp: 100,
        quality: 4,
        ..Default::default()
    });
    let rows = vec![
        DcNetDataSellRow {
            user_dat_id: 7001,
            item_id: equip_id,
            ..Default::default()
        },
        DcNetDataSellRow {
            user_dat_id: 8001,
            item_id: partner_id,
            ..Default::default()
        },
    ];

    let result = player.disassemble(&rows, &tables).unwrap();

    assert_eq!(result.sold, rows);
    assert!(
        player
            .equips
            .iter()
            .all(|equip| equip.user_equip_id != 7001)
    );
    assert!(player.partners.iter().all(|partner| partner.id != 8001));
    assert_eq!(
        result
            .reward
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [
            (100_100_027, 1),
            (100_100_028, 1),
            (100_300_004, 1),
            (100_300_007, 1),
        ]
    );
}

#[test]
fn invalid_disassembly_is_atomic() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let equip_id = tables.equipment.rows[0].id;
    player.equips.push(DcNetDataEquip {
        equip_id,
        user_equip_id: 7001,
        locked: 1,
        ..Default::default()
    });
    let before = player.equips.clone();

    assert_eq!(
        player.disassemble(
            &[DcNetDataSellRow {
                user_dat_id: 7001,
                item_id: equip_id,
                ..Default::default()
            }],
            &tables,
        ),
        Err(DisassemblyError::Locked(7001))
    );
    assert_eq!(player.equips, before);
    assert!(player.items.iter().all(|item| item.item_id != 100_100_028));
}
