use super::*;

#[test]
fn equipment_groups_save_update_apply_delete_and_persist() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let roles = player
        .roles
        .iter()
        .take(2)
        .map(|role| role.role_basic_info.as_ref().unwrap().game_role_id)
        .collect::<Vec<_>>();
    player.equips.extend([
        DcNetDataEquip {
            user_equip_id: 1,
            pos: 1,
            ..Default::default()
        },
        DcNetDataEquip {
            user_equip_id: 2,
            pos: 2,
            ..Default::default()
        },
        DcNetDataEquip {
            user_equip_id: 3,
            pos: 1,
            ..Default::default()
        },
    ]);

    let (saved, changed) = player
        .save_equipment_group(DcNetDataEquipsGroup {
            id: 0,
            game_role_id: roles[0],
            group_name: "boss".into(),
            user_equip_ids: vec![1, 2],
        })
        .unwrap();
    assert_eq!(saved.id, 1);
    assert_eq!(changed.len(), 2);
    assert_eq!(player.equips[0].in_group, 1);

    let (_, changed) = player
        .save_equipment_group(DcNetDataEquipsGroup {
            id: saved.id,
            game_role_id: roles[0],
            group_name: "boss v2".into(),
            user_equip_ids: vec![3, 2],
        })
        .unwrap();
    assert_eq!(changed.len(), 2);
    assert_eq!(
        (player.equips[0].in_group, player.equips[2].in_group),
        (0, 1)
    );

    let results = player.set_equipment_group(roles[1], &[3, 2]).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|result| {
        result
            .equip_info
            .as_ref()
            .is_some_and(|equip| equip.equiped_role == roles[1])
    }));
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.equipment_groups, player.equipment_groups);
    assert_eq!(restored.equips, player.equips);

    assert_eq!(
        player.save_equipment_group(DcNetDataEquipsGroup {
            id: saved.id,
            game_role_id: roles[0],
            group_name: "invalid".into(),
            user_equip_ids: vec![1, 3],
        }),
        Err(EquipmentError::DuplicateGroupSlot(1))
    );
    player.delete_equipment_group(saved.id).unwrap();
    assert!(player.equipment_groups.is_empty());
    assert!(player.equips.iter().all(|equip| equip.in_group == 0));
}
fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn equipment_set_unset_swap_and_lock_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let roles: Vec<_> = player
        .roles
        .iter()
        .take(2)
        .map(|role| role.role_basic_info.as_ref().unwrap().game_role_id)
        .collect();
    player.equips.extend([
        DcNetDataEquip {
            equip_id: 100_500_114,
            user_equip_id: 1,
            pos: 1,
            ..Default::default()
        },
        DcNetDataEquip {
            equip_id: 100_500_115,
            user_equip_id: 2,
            pos: 1,
            ..Default::default()
        },
    ]);

    let first = player.set_equipment(roles[0], 1, 1).unwrap();
    assert_eq!(first.equip_info.unwrap().equiped_role, roles[0]);
    assert!(first.dropped_list.is_empty());
    let second = player.set_equipment(roles[0], 2, 1).unwrap();
    assert_eq!(second.dropped_list[0].user_equip_id, 1);
    player.set_equipment(roles[1], 1, 1).unwrap();

    let (first, second) = player.swap_equipment(1, 2).unwrap();
    assert_eq!(
        (first.equiped_role, second.equiped_role),
        (roles[0], roles[1])
    );
    player.lock_equipment(1, 1).unwrap();
    assert_eq!(player.unset_equipment(roles[0], 1).unwrap()[0].locked, 1);

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.equips, player.equips);
}

#[test]
fn equipment_level_up_uses_extracted_exp_tables_and_persists() {
    let (mut tables, config) = fixture();
    tables.cultivation_constants.coin_item_id = 100_100_002;
    tables
        .items
        .rows
        .iter_mut()
        .find(|item| item.id == tables.cultivation_constants.equipment_exp_items[0])
        .unwrap()
        .effect
        .as_mut()
        .unwrap()
        .key = 123_456;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let material_id = tables.cultivation_constants.equipment_exp_items[0];
    let coin_item_id = tables.cultivation_constants.coin_item_id;
    let material = player.add_item(material_id, 2, &tables).unwrap();
    player.add_item(coin_item_id, 1_000, &tables).unwrap();
    let gold_before = player
        .items
        .iter()
        .find(|item| item.item_id == coin_item_id)
        .unwrap()
        .amount;
    let mut relic = DcNetDataEquip {
        equip_id: 100_500_114,
        user_equip_id: 1,
        pos: 1,
        quality: 4,
        ..Default::default()
    };
    equipment::initialize_equipment_words(&mut relic, player.uid, &tables);
    player.equips.push(relic);
    let use_one = DcNetDataUseItem {
        user_item_id: material.user_item_id,
        item_id: material_id,
        amount: 1,
        ..Default::default()
    };

    let (equip, changed, returns) = player.level_up_equipment(1, &[use_one], &tables).unwrap();
    assert_eq!((equip.level, equip.exp), (0, 500));
    assert_eq!(changed[0].amount, 1);
    assert!(returns.is_empty());

    let (equip, changed, _) = player.level_up_equipment(1, &[use_one], &tables).unwrap();
    assert_eq!((equip.level, equip.exp), (1, 400));
    assert_eq!(changed[0].amount, 0);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == coin_item_id)
            .unwrap()
            .amount,
        gold_before - 300
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.equips, player.equips);
    assert_eq!(restored.items, player.items);
}

#[test]
fn equipment_random_word_reroll_uses_quality_cost_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let cost = tables.equipment_parameters.get(4).unwrap().extra_word_costs[0].clone();
    player.add_item(cost.key, cost.value, &tables).unwrap();
    let mut relic = DcNetDataEquip {
        equip_id: 100_500_114,
        user_equip_id: 1,
        level: 4,
        pos: 1,
        quality: 4,
        random_words_id: vec![1],
        tmp_word_idx: -1,
        ..Default::default()
    };
    equipment::initialize_equipment_words(&mut relic, player.uid, &tables);
    player.equips.push(relic);

    let (word_id, remains) = player.refresh_equipment_word(1, 0, &tables).unwrap();
    assert!(
        tables
            .extra_equipment_words_by_slot
            .get(1)
            .unwrap()
            .any(|word| word.id == word_id)
    );
    assert_eq!(remains[0].amount, 0);
    assert_eq!(
        (player.equips[0].tmp_word, player.equips[0].minnum),
        (word_id, 1)
    );

    let equip = player
        .replace_equipment_word(1, 0, 0, true, &tables)
        .unwrap();
    assert_eq!(equip.random_words_id, [word_id]);
    assert_eq!(
        (equip.tmp_word, equip.tmp_word_idx, equip.minnum),
        (0, -1, 0)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.equips, player.equips);
}

#[test]
fn equipment_generation_uses_table_roll_count_and_breakpoints_respect_the_cap() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let mut equip = DcNetDataEquip {
        equip_id: 100_500_114,
        user_equip_id: 1,
        level: 0,
        pos: 1,
        quality: 4,
        tmp_word_idx: -1,
        ..Default::default()
    };
    equipment::initialize_equipment_words(&mut equip, player.uid, &tables);
    assert!(
        tables
            .main_equipment_words_by_group
            .get(1001)
            .unwrap()
            .any(|word| word.id == equip.main_words_id)
    );
    assert_eq!(equip.deputy_words_id.len(), 2);
    assert!(equip.random_words_id.is_empty());

    equip.level = 3;
    player.equips.push(equip);
    let material_id = tables.cultivation_constants.equipment_exp_items[0];
    let coin_item_id = tables.cultivation_constants.coin_item_id;
    let material = player.add_item(material_id, 2, &tables).unwrap();
    player.add_item(coin_item_id, 1_000, &tables).unwrap();
    let (equip, _, _) = player
        .level_up_equipment(
            1,
            &[DcNetDataUseItem {
                user_item_id: material.user_item_id,
                item_id: material_id,
                amount: 2,
                ..Default::default()
            }],
            &tables,
        )
        .unwrap();
    assert_eq!(equip.level, 4);
    assert_eq!(equip.random_words_id.len(), 1);
    assert_eq!(equip.deputy_words_id.len(), 2);
}

#[test]
fn generated_relics_roll_per_instance_and_custom_stats_are_validated() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let relic_id = 100_500_116;

    let random = player.generate_relics(relic_id, 2, &tables, 123).unwrap();
    assert_ne!(random[0].user_equip_id, random[1].user_equip_id);
    assert_ne!(random[0].deputy_words_id, random[1].deputy_words_id);
    assert!(
        random
            .iter()
            .all(|relic| relic.deputy_words_id.len() == 6 && relic.random_words_id.is_empty())
    );

    let relic = tables.equipment.get(relic_id).unwrap();
    let main_stat = tables
        .main_equipment_words_by_group
        .get(relic.group_id)
        .unwrap()
        .next()
        .unwrap()
        .id;
    let substats = tables
        .extra_equipment_words_by_slot
        .get(relic.slot)
        .unwrap()
        .take(6)
        .map(|word| word.id)
        .collect::<Vec<_>>();
    let custom = player
        .generate_custom_relics(relic_id, 1, main_stat, &substats, &tables, 124)
        .unwrap();
    assert_eq!(custom[0].main_words_id, main_stat);
    assert_eq!(custom[0].deputy_words_id, substats);
    assert!(custom[0].random_words_id.is_empty());
    assert!(
        !player
            .equipment_word_plans
            .contains_key(&custom[0].user_equip_id)
    );

    let with_plus_one = vec![
        substats[0],
        substats[0],
        substats[1],
        substats[2],
        substats[3],
        substats[4],
    ];
    let mut custom = player
        .generate_custom_relics(relic_id, 1, main_stat, &with_plus_one, &tables, 125)
        .unwrap();
    assert_eq!(custom[0].deputy_words_id, with_plus_one);
    assert!(custom[0].random_words_id.is_empty());
    assert!(
        !player
            .equipment_word_plans
            .contains_key(&custom[0].user_equip_id)
    );
    equipment::append_unlocked_words(&mut custom[0], 0, 20, 42, &tables);
    assert_eq!(custom[0].random_words_id.len(), 5);
    assert!(
        custom[0]
            .random_words_id
            .iter()
            .all(|word| with_plus_one.contains(word))
    );

    let repeated = vec![substats[0]; 6];
    assert!(matches!(
        player.generate_custom_relics(relic_id, 1, main_stat, &repeated, &tables, 125),
        Err(EquipmentError::RelicSubstatLimit { .. })
    ));
    assert!(matches!(
        player.generate_custom_relics(relic_id, 1, -1, &substats, &tables, 125),
        Err(EquipmentError::InvalidRelicMainStat { .. })
    ));
    assert!(matches!(
        player.generate_custom_relics(relic_id, 1, main_stat, &substats[..5], &tables, 125),
        Err(EquipmentError::InvalidRelicSubstatCount { .. })
    ));
    assert_eq!(
        player.generate_relics(100_500_846, 1, &tables, 126),
        Err(EquipmentError::UnavailableRelic(100_500_846))
    );
}

#[test]
fn gold_relics_start_with_six_rolls_and_level_rolls_improve_existing_stats() {
    let (tables, _) = fixture();
    let mut equip = DcNetDataEquip {
        equip_id: 100_500_116,
        user_equip_id: 1,
        pos: 3,
        quality: 6,
        tmp_word_idx: -1,
        ..Default::default()
    };
    equipment::initialize_equipment_words(&mut equip, 42, &tables);
    assert_eq!(equip.deputy_words_id.len(), 6);
    assert!(equip.random_words_id.is_empty());
    let initial_effects = equip
        .deputy_words_id
        .iter()
        .map(|word| tables.extra_equipment_words.get(*word).unwrap().effect.key)
        .collect::<std::collections::HashSet<_>>();
    equipment::append_unlocked_words(&mut equip, 0, 20, 42, &tables);
    assert_eq!(equip.deputy_words_id.len(), 6);
    assert_eq!(equip.random_words_id.len(), 5);
    assert!(equip.random_words_id.iter().all(|word| {
        initial_effects.contains(&tables.extra_equipment_words.get(*word).unwrap().effect.key)
    }));

    let first = equip.deputy_words_id[0];
    let pending_word = equip.random_words_id.first().copied().unwrap_or(first);
    let mut legacy = DcNetDataEquip {
        equip_id: equip.equip_id,
        user_equip_id: 4,
        level: 20,
        pos: equip.pos,
        quality: equip.quality,
        deputy_words_id: vec![first; 6],
        random_words_id: equip.random_words_id.clone(),
        tmp_word: pending_word,
        tmp_word_idx: 8,
        ..Default::default()
    };
    let mut legacy_plan = Vec::new();
    equipment::normalize_equipment_words(&mut legacy, &mut legacy_plan, &tables);
    assert_eq!(legacy.deputy_words_id.len(), 6);
    assert!(legacy.random_words_id.len() <= 5);
    assert!(legacy_plan.is_empty());
    assert_eq!((legacy.tmp_word, legacy.tmp_word_idx), (0, -1));
}
