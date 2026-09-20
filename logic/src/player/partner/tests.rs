use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

fn partner(id: i64) -> DcNetDataPartner {
    DcNetDataPartner {
        id,
        lv: 1,
        partner_id: 100_400_026,
        quality: 5,
        reson_lv: 1,
        skill_lv: 1,
        ..Default::default()
    }
}

#[test]
fn grant_all_partners_only_adds_missing_definitions() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let rewards = player.grant_all_partners(&tables, 123);
    assert_eq!(player.partners.len(), tables.partners.rows.len());
    assert_eq!(rewards.partner_rewards.len(), tables.partners.rows.len());
    assert!(
        player
            .grant_all_partners(&tables, 124)
            .partner_rewards
            .is_empty()
    );
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.partners.len(), tables.partners.rows.len());
}

#[test]
fn duplicate_partner_grants_create_resonance_material() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let partner_id = 100_463_900;

    let rewards = player.grant_partner(partner_id, 3, &tables, 123);

    assert_eq!(rewards.partner_rewards.len(), 3);
    let owned = player
        .partners
        .iter()
        .filter(|partner| partner.partner_id == partner_id)
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 3);
    assert_ne!(owned[0].id, owned[1].id);
    assert_ne!(owned[1].id, owned[2].id);
}

#[test]
fn gm_max_partner_uses_every_table_defined_cap() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let partner = partner(1);
    player.partners.push(partner);

    let (maxed, changed) = player.max_partner(partner.partner_id, &tables).unwrap();

    assert!(changed);
    assert_eq!((maxed.lv, maxed.exp), (70, 0));
    assert_eq!(maxed.brek, 5);
    assert_eq!(maxed.skill_lv, 4);
    assert_eq!(maxed.reson_lv, 4);
}

#[test]
fn partner_progression_uses_extracted_cost_tables_and_persists() {
    let (mut tables, config) = fixture();
    tables.cultivation_constants.coin_item_id = 100_100_002;
    tables
        .items
        .rows
        .iter_mut()
        .find(|item| item.id == 100_300_004)
        .unwrap()
        .effect
        .as_mut()
        .unwrap()
        .key = 123_456;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .add_item(tables.cultivation_constants.coin_item_id, 500_000, &tables)
        .unwrap();
    player.add_item(100_100_003, 500_000, &tables).unwrap();
    for (item_id, amount) in [(100_300_004, 1), (100_301_007, 6), (100_301_205, 4)] {
        player.add_item(item_id, amount, &tables).unwrap();
    }
    player.partners.extend([partner(1), partner(2)]);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_703_000_001,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });

    let leveled = player
        .level_up_partner(
            1,
            &[DcNetDataItem {
                item_id: 100_300_004,
                amount: 1,
                ..Default::default()
            }],
            &tables,
        )
        .unwrap();
    assert_eq!((leveled.partner.lv, leveled.partner.exp), (3, 520));
    assert_eq!(
        leveled
            .cost_remain
            .iter()
            .find(|item| item.item_id == 100_300_004)
            .unwrap()
            .amount,
        0
    );
    assert_eq!(leveled.remain[0].item_id, 123_456);
    assert!(
        leveled
            .cost_remain
            .iter()
            .any(|item| item.item_id == tables.cultivation_constants.coin_item_id)
    );

    player.partners[0].lv = 20;
    let (broken, _) = player.break_partner(1, &tables).unwrap();
    assert_eq!(broken.brek, 1);
    let (skilled, _) = player.upgrade_partner_skill(1, &tables).unwrap();
    assert_eq!(skilled.skill_lv, 2);
    let resonated = player.resonate_partner(1, &[2], &tables).unwrap();
    assert_eq!(resonated.partner.reson_lv, 2);
    assert_eq!(resonated.consumed_partners, [2]);
    assert_eq!(player.partners.len(), 1);

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.partners, player.partners);
    assert_eq!(restored.items, player.items);
}

#[test]
fn partner_level_up_accepts_zero_count_material_slots_sent_by_client() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.partners.push(partner(1));
    player
        .add_item(tables.cultivation_constants.coin_item_id, 10_000, &tables)
        .unwrap();
    player.add_item(100_300_005, 1, &tables).unwrap();

    let outcome = player
        .level_up_partner(
            1,
            &[
                DcNetDataItem {
                    item_id: 100_300_004,
                    amount: 0,
                    ..Default::default()
                },
                DcNetDataItem {
                    item_id: 100_300_005,
                    amount: 1,
                    ..Default::default()
                },
                DcNetDataItem {
                    item_id: 100_300_006,
                    amount: 0,
                    ..Default::default()
                },
            ],
            &tables,
        )
        .unwrap();

    assert!(outcome.partner.lv > 1);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_300_005)
            .unwrap()
            .amount,
        0
    );
}

#[test]
fn partner_progression_rejects_invalid_costs_without_mutating_state() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.partners.extend([partner(1), partner(2)]);
    player.partners[0].lv = 20;
    player.partners[1].locked = 1;
    let before = player.to_record();

    assert_eq!(
        player.break_partner(1, &tables).unwrap_err(),
        PartnerProgressError::WorldLevelRequired {
            needed: 1,
            current: 0,
        }
    );
    assert_eq!(
        player.resonate_partner(1, &[2], &tables).unwrap_err(),
        PartnerProgressError::LockedResonancePartner(2)
    );
    assert_eq!(player.to_record(), before);
}
