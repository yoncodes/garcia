use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    (tables, config)
}

#[test]
fn team_equipment_lifecycle_uses_table_slots_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let formation_id = player.formations[0].formation_id;
    let first = player.grant_team_equip(100_800_011, &tables).unwrap();
    let second = player.grant_team_equip(100_800_021, &tables).unwrap();

    assert_eq!(first.main_words_id, 10_080_116);
    assert_eq!(first.pos_num, 3);
    assert_ne!(first.id, second.id);
    assert_eq!(
        player
            .equip_team_equip(first.id, formation_id, &tables)
            .unwrap(),
        (
            DcNetDataTeamEquip {
                equiped_fid: formation_id,
                ..first
            },
            0,
        )
    );
    player
        .equip_team_equip(second.id, formation_id, &tables)
        .unwrap();
    assert_eq!(
        player
            .team_equips
            .iter()
            .find(|equip| equip.id == first.id)
            .unwrap()
            .equiped_fid,
        0
    );
    assert!(player.lock_team_equip(second.id, true).unwrap().locked);

    player.team_cores.extend([
        DcNetDataTeamCore {
            id: 1,
            core_id: 100_900_001,
            ..Default::default()
        },
        DcNetDataTeamCore {
            id: 2,
            core_id: 100_900_002,
            ..Default::default()
        },
    ]);
    let (attached, dropped) = player
        .set_team_equip_core(second.id, 1, 1, &tables)
        .unwrap();
    assert_eq!((attached.equiped_id, attached.pos), (second.id, 1));
    assert_eq!(dropped, None);
    let (_, dropped) = player
        .set_team_equip_core(second.id, 2, 1, &tables)
        .unwrap();
    assert_eq!(dropped.unwrap().id, 1);
    assert_eq!(player.team_equip_details()[1].core_info.get(&1), Some(&2));
    assert_eq!(
        player
            .unset_team_equip_core(second.id, 1)
            .unwrap()
            .unwrap()
            .id,
        2
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.team_equips, player.team_equips);
    assert_eq!(restored.team_cores, player.team_cores);
}

#[test]
fn invalid_team_equipment_requests_are_atomic() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let equip = player.grant_team_equip(100_800_011, &tables).unwrap();
    player.team_cores.push(DcNetDataTeamCore {
        id: 1,
        core_id: 100_900_001,
        ..Default::default()
    });
    let before = player.team_cores.clone();

    assert_eq!(
        player.set_team_equip_core(equip.id, 1, equip.pos_num + 1, &tables),
        Err(TeamEquipError::InvalidCorePosition {
            equip_id: equip.id,
            position: equip.pos_num + 1,
        })
    );
    assert_eq!(player.team_cores, before);
    assert_eq!(
        player.equip_team_equip(equip.id, i32::MAX, &tables),
        Err(TeamEquipError::UnknownFormation(i32::MAX))
    );
    assert_eq!(
        player.lock_team_equip(i64::MAX, true),
        Err(TeamEquipError::UnknownEquip(i64::MAX))
    );
}

#[test]
fn team_equipment_level_up_consumes_table_exp_sources() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let target = player.grant_team_equip(100_800_011, &tables).unwrap();
    let material = player.grant_team_equip(100_800_015, &tables).unwrap();
    player.items.retain(|item| item.item_id != 100_300_007);
    player.items.push(DcNetDataItem {
        user_item_id: 77,
        item_id: 100_300_007,
        amount: 1,
        quality: 3,
        ..Default::default()
    });

    let (upgraded, remains) = player
        .level_up_team_equip(
            target.id,
            &[DcNetDataItem {
                user_item_id: 77,
                item_id: 100_300_007,
                amount: 1,
                quality: 3,
                ..Default::default()
            }],
            &[material.id],
            &tables,
        )
        .unwrap();

    assert_eq!((upgraded.lv, upgraded.exp), (3, 50));
    assert_eq!(remains[0].amount, 0);
    assert!(
        player
            .team_equips
            .iter()
            .all(|equip| equip.id != material.id)
    );
}
