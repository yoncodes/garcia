use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn gm_progression_uses_player_exp_world_tasks_and_story_ports() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let level = player.increase_player_level(20, &tables, 122).unwrap();
    assert_eq!(player.level, 20);
    assert!(level.level_update.is_some_and(|update| update.is_lv_up));

    let world = player.increase_world_level(2, &tables, 123).unwrap();
    assert_eq!(player.level, 30);
    assert_eq!(player.world_level(&tables), 2);
    assert!(
        world
            .ports
            .iter()
            .any(|port| port.id == 180_101_189 && port.pass_cnt == 1)
    );

    let stage = player
        .complete_story_stage(180_101_056, &tables, 124)
        .unwrap();
    assert!(
        stage
            .ports
            .iter()
            .any(|port| port.id == 180_101_056 && port.is_c)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.level, 30);
    assert_eq!(restored.world_level(&tables), 2);
    assert!(
        restored
            .ports
            .iter()
            .any(|port| port.id == 180_101_056 && port.pass_cnt == 1)
    );
}

#[test]
fn player_level_61_waits_for_world_level_five() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    player.increase_world_level(4, &tables, 123).unwrap();
    player.increase_player_level(61, &tables, 123).unwrap();
    assert_eq!(player.level, 60);
    assert_eq!(player.world_level(&tables), 4);

    let unlocked = player.increase_world_level(5, &tables, 124).unwrap();
    assert_eq!(player.world_level(&tables), 5);
    assert_eq!(player.level, 61);
    assert_eq!(unlocked.level_update.unwrap().attr_lv, 61);
    assert!(
        unlocked
            .mission_updates
            .iter()
            .any(|mission| mission.misson_id == 61 && mission.curr_num >= 61)
    );

    player.level = 70;
    let mission = player
        .missions
        .iter_mut()
        .find(|mission| mission.misson_id == 61)
        .unwrap();
    mission.curr_num = 60;
    mission.taken = false;
    let repaired = player.increase_world_level(5, &tables, 125).unwrap();
    assert!(
        repaired
            .mission_updates
            .iter()
            .any(|mission| mission.misson_id == 61 && mission.curr_num == 70)
    );
}

#[test]
fn grant_all_roles_only_adds_missing_roles() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let initially_owned = player.roles.len();
    let playable_roles = tables
        .maids
        .rows
        .iter()
        .filter(|maid| tables.is_playable_maid(maid.id))
        .count();

    let rewards = player.grant_all_roles(&tables, 123);
    assert_eq!(player.roles.len(), playable_roles);
    assert_eq!(rewards.role_rewards.len(), playable_roles - initially_owned);
    for role in &player.roles {
        let maid_id = role.role_basic_info.as_ref().unwrap().game_role_id;
        if let Some(avatar) = tables.profile_avatar_for_maid(maid_id) {
            assert!(
                player
                    .profile_avatars
                    .iter()
                    .any(|owned| owned.id == avatar.id)
            );
        }
    }
    assert!(
        rewards
            .profileavatar_rewards
            .iter()
            .all(|avatar| player.profile_avatars.contains(avatar))
    );
    assert!(player.grant_all_roles(&tables, 124).role_rewards.is_empty());
    let missing_avatar = rewards.profileavatar_rewards.first().unwrap().id;
    let mut record = player.to_record();
    record
        .profile_unlocks
        .retain(|profile| profile.profile_id != missing_avatar);
    let restored = Player::from_record(record, &tables, &config.account_defaults);
    assert_eq!(restored.roles.len(), playable_roles);
    assert!(
        restored
            .profile_avatars
            .iter()
            .any(|avatar| avatar.id == missing_avatar)
    );
    assert!(
        [110_100_004, 110_100_011, 110_100_020]
            .into_iter()
            .all(|id| !restored.owns_role(id))
    );
}

#[test]
fn duplicate_roles_grant_fragments_then_overflow_currency() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_005;
    let maid = tables.maids.get(role_id).unwrap();

    let first = player.grant_role(role_id, &tables, 123);
    assert_eq!(first.role_rewards.len(), 1);
    assert!(first.role_rewards[0].on_duplicate.is_none());

    let duplicate = player.grant_role(role_id, &tables, 124);
    assert_eq!(
        duplicate.role_rewards[0]
            .on_duplicate
            .map(|item| (item.item_id, item.amount)),
        Some((maid.duplicate_reward.key, maid.duplicate_reward.value))
    );

    player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap()
        .maid_qua = tables.cultivation_constants.max_role_resonance - 1;
    let overflow = player.grant_role(role_id, &tables, 125);
    assert_eq!(
        overflow.role_rewards[0]
            .on_duplicate
            .map(|item| (item.item_id, item.amount)),
        Some((100_100_023, 45))
    );
    assert_eq!(
        player
            .roles
            .iter()
            .filter(|role| role
                .role_basic_info
                .as_ref()
                .is_some_and(|role| role.game_role_id == role_id))
            .count(),
        1
    );
}

#[test]
fn loading_discards_non_playable_role_records() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .roles
        .push(gacha_role(player.uid, 110_100_004, &tables).unwrap());

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);

    assert!(!restored.owns_role(110_100_004));
}

#[test]
fn role_rank_resonance_and_rewards_use_extracted_tables_and_persist() {
    let (mut tables, config) = fixture();
    tables.cultivation_constants.coin_item_id = 100_100_002;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    let role = player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap();
    role.level = 20;
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_703_000_001,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    for (item_id, amount) in [
        (100_301_007, 10),
        (100_307_004, 5),
        (100_100_003, 40_000),
        (tables.cultivation_constants.coin_item_id, 44_000),
        (100_200_013, 1),
        (100_301_107, 14),
    ] {
        player.add_item(item_id, amount, &tables).unwrap();
    }

    let ranked = player.rank_up_role(role_id, &tables).unwrap();
    assert_eq!((ranked.game_role_id, ranked.position), (role_id, 1));
    assert_eq!(ranked.items.len(), 3);
    let reward = player
        .claim_role_rank_reward(role_id, 0, &tables, 1_787_522_872)
        .unwrap();
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (100_100_002, 44_030)
    );
    assert_eq!(
        player
            .claim_role_rank_reward(role_id, 0, &tables, 1_787_522_872)
            .unwrap_err(),
        RoleMutationError::RankRewardClaimed { role_id, level: 0 }
    );
    let (resonance, remains, _) = player.resonate_role(role_id, &tables).unwrap();
    assert_eq!(resonance, 1);
    assert_eq!((remains[0].item_id, remains[0].amount), (100_200_013, 0));
    let (talent, remains) = player.level_up_role_talent(role_id, 1, &tables).unwrap();
    assert_eq!((talent.position, talent.lv), (1, 2));
    assert_eq!(remains.len(), 2);
    assert!(
        player
            .items
            .iter()
            .find(|item| item.item_id == tables.cultivation_constants.coin_item_id)
            .unwrap()
            .amount
            < 44_030
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    let restored_role = restored
        .roles
        .iter()
        .find(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|role| role.game_role_id == role_id)
        })
        .unwrap();
    let restored_info = restored_role.role_basic_info.as_ref().unwrap();
    assert_eq!((restored_info.position, restored_info.maid_qua), (1, 1));
    assert_eq!(restored_info.awards, [0]);
    assert_eq!(
        restored_role
            .talents
            .iter()
            .find(|talent| talent.position == 1)
            .unwrap()
            .lv,
        2
    );
}

#[test]
fn max_resonance_unlocks_and_restores_the_roles_namecard() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    let namecard_id = tables.profile_card_for_maid(role_id).unwrap().id;
    let resonance_cost = tables.maids.get(role_id).unwrap().quality_break.clone();
    player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap()
        .maid_qua = tables.cultivation_constants.max_role_resonance - 1;
    player
        .add_item(resonance_cost.key, resonance_cost.value, &tables)
        .unwrap();

    let (_, _, namecard) = player.resonate_role(role_id, &tables).unwrap();

    assert_eq!(namecard.map(|card| card.id), Some(namecard_id));
    assert!(
        player
            .profile_cards
            .iter()
            .any(|card| card.id == namecard_id)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert!(
        restored
            .profile_cards
            .iter()
            .any(|card| card.id == namecard_id)
    );
}

#[test]
fn loading_repairs_missing_namecards_for_already_maxed_roles() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    let namecard_id = tables.profile_card_for_maid(role_id).unwrap().id;
    player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap()
        .maid_qua = tables.cultivation_constants.max_role_resonance;

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);

    assert!(
        restored
            .profile_cards
            .iter()
            .any(|card| card.id == namecard_id)
    );
}

#[test]
fn gm_max_role_uses_every_table_defined_cap() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;

    let outcome = player.max_role(role_id, &tables).unwrap();
    let info = outcome.role.role_basic_info.unwrap();

    assert!(outcome.changed);
    assert_eq!(
        info.maid_qua,
        tables.cultivation_constants.max_role_resonance
    );
    assert_eq!(info.position, 5);
    assert_eq!(info.level, 70);
    assert_eq!(info.exp, 0);
    assert!(outcome.role.talents.iter().all(|talent| {
        talent.lv
            == tables
                .maid_talent_levels
                .iter()
                .filter(|level| level.quality == 5 && level.position == talent.position)
                .map(|level| level.level)
                .max()
                .unwrap()
    }));
    assert_eq!(
        outcome.namecard.map(|card| card.id),
        tables.profile_card_for_maid(role_id).map(|card| card.id)
    );
}

#[test]
fn role_talent_enforces_the_extracted_rank_gate() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    assert_eq!(
        player
            .level_up_role_talent(role_id, 7, &tables)
            .unwrap_err(),
        RoleMutationError::TalentRankRequired {
            position: 7,
            needed: 2,
            current: 0,
        }
    );
}

#[test]
fn role_rank_requires_the_configured_world_level() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap()
        .level = 20;
    assert_eq!(
        player.rank_up_role(role_id, &tables).unwrap_err(),
        RoleMutationError::RankWorldLevelRequired {
            needed: 1,
            current: 0,
        }
    );
}
#[test]
fn port_settlement_uses_gameplay_and_level_tables() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;

    player.start_port(180_101_056, &tables).unwrap();
    let (port, roles) = player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_056,
                is_completed: 1,
                role_data: vec![protocol::pbcommon::DcNetDataRoleData {
                    game_role_id: role_id,
                    ..Default::default()
                }],
                stars: vec![true, false, true],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();

    assert_eq!(player.gameplay_id, 180_101_056);
    assert_eq!(port.port_id, 140_301_056);
    assert_eq!((port.pass_cnt, port.all_cnt), (1, 1));
    assert_eq!((port.s1, port.s2, port.s3), (1, 0, 1));
    let level = roles[0].lv_up_data.as_ref().unwrap();
    assert_eq!((level.attr_lv, level.attr_exp, level.add_exp), (2, 60, 100));
    assert!(level.is_lv_up);
    assert_eq!(
        (level.items[0].item_id, level.items[0].amount),
        (100_100_005, 60)
    );

    player.start_port(180_101_057, &tables).unwrap();
    let (_, roles) = player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_057,
                is_completed: 1,
                role_data: vec![protocol::pbcommon::DcNetDataRoleData {
                    game_role_id: role_id,
                    ..Default::default()
                }],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();
    let level = roles[0].lv_up_data.as_ref().unwrap();
    assert_eq!((level.attr_lv, level.attr_exp), (3, 20));
}

#[test]
fn port_settlement_records_unique_monsters_and_persists_them() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let monster_hashes = tables
        .monsters
        .rows
        .iter()
        .take(3)
        .map(|monster| monster.id_crc)
        .collect::<Vec<_>>();

    player.start_port(180_101_070, &tables).unwrap();
    player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_070,
                monsters: vec![
                    protocol::pbcommon::DcNetDataMonsterKilledInfo {
                        enemy_hash: monster_hashes[0],
                        amount: 1,
                    },
                    protocol::pbcommon::DcNetDataMonsterKilledInfo {
                        enemy_hash: monster_hashes[1],
                        amount: 4,
                    },
                    protocol::pbcommon::DcNetDataMonsterKilledInfo {
                        enemy_hash: monster_hashes[0],
                        amount: 1,
                    },
                ],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();

    player.start_port(180_101_070, &tables).unwrap();
    player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_070,
                monsters: vec![
                    protocol::pbcommon::DcNetDataMonsterKilledInfo {
                        enemy_hash: monster_hashes[1],
                        amount: 1,
                    },
                    protocol::pbcommon::DcNetDataMonsterKilledInfo {
                        enemy_hash: monster_hashes[2],
                        amount: 1,
                    },
                ],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();

    assert_eq!(player.monster_manuals.len(), 1);
    assert_eq!(player.monster_manuals[0].port_id, 180_101_070);
    assert_eq!(player.monster_manuals[0].mons, monster_hashes);

    let mut record = player.to_record();
    record
        .monster_manuals
        .push(database::models::game::player_state::MonsterManualRecord {
            gameplay_id: 180_101_070,
            enemy_hash: u32::MAX,
        });
    let restored = Player::from_record(record, &tables, &config.account_defaults);
    assert_eq!(restored.monster_manuals, player.monster_manuals);
}

#[test]
fn port_settlement_rejects_unknown_monster_before_mutating_player() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let unknown_crc = u32::MAX;
    assert!(
        tables
            .monsters
            .rows
            .iter()
            .all(|monster| monster.id_crc != unknown_crc)
    );
    player.start_port(180_101_070, &tables).unwrap();
    let ports = player.ports.clone();

    let error = player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_070,
                monsters: vec![protocol::pbcommon::DcNetDataMonsterKilledInfo {
                    enemy_hash: unknown_crc,
                    amount: 1,
                }],
                ..Default::default()
            },
            &tables,
        )
        .unwrap_err();

    assert_eq!(error, PortError::UnknownMonsterCrc(unknown_crc));
    assert_eq!(player.ports, ports);
    assert!(player.monster_manuals.is_empty());
}

#[test]
fn port_settlement_does_not_store_exp_past_the_role_level_cap() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role = player.roles[0].role_basic_info.as_mut().unwrap();
    let role_id = role.game_role_id;
    role.level = 20;
    role.exp = 0;

    player.start_port(180_101_070, &tables).unwrap();
    let (_, roles) = player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_070,
                is_completed: 1,
                role_data: vec![protocol::pbcommon::DcNetDataRoleData {
                    game_role_id: role_id,
                    ..Default::default()
                }],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();

    let level = roles[0].lv_up_data.as_ref().unwrap();
    assert_eq!((level.attr_lv, level.attr_exp, level.add_exp), (20, 0, 100));
    assert_eq!(level.items[0].amount, 0);
    assert!(!level.is_lv_up);
    assert_eq!(player.roles[0].role_basic_info.as_ref().unwrap().exp, 0);

    let required_exp = tables.maid_levels.get(19).unwrap().exp;
    let role = player.roles[0].role_basic_info.as_mut().unwrap();
    role.level = 19;
    role.exp = required_exp - 50;
    player.start_port(180_101_071, &tables).unwrap();
    let (_, roles) = player
        .settle_port(
            &DcNetDataParamsSettlement {
                id: 180_101_071,
                is_completed: 1,
                role_data: vec![protocol::pbcommon::DcNetDataRoleData {
                    game_role_id: role_id,
                    ..Default::default()
                }],
                ..Default::default()
            },
            &tables,
        )
        .unwrap();
    let level = roles[0].lv_up_data.as_ref().unwrap();
    assert_eq!((level.attr_lv, level.attr_exp), (20, 0));
    assert!(level.is_lv_up);
}

#[test]
fn role_level_up_clears_stored_exp_at_the_rank_cap() {
    let (mut tables, config) = fixture();
    tables.cultivation_constants.coin_item_id = 100_100_002;
    tables
        .items
        .rows
        .iter_mut()
        .find(|item| item.id == 100_300_001)
        .unwrap()
        .effect
        .as_mut()
        .unwrap()
        .key = 100_100_006;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role = player.roles[0].role_basic_info.as_mut().unwrap();
    let role_id = role.game_role_id;
    role.level = 19;
    role.exp = 0;
    player.add_item(100_300_001, 100, &tables).unwrap();
    player
        .add_item(
            tables.cultivation_constants.coin_item_id,
            1_000_000,
            &tables,
        )
        .unwrap();

    let outcome = player
        .level_up_role(
            role_id,
            &[DcNetDataUseItem {
                item_id: 100_300_001,
                amount: 100,
                ..Default::default()
            }],
            &tables,
            1_787_523_760,
            config.server.zone_offset,
        )
        .unwrap();

    assert_eq!(outcome.user_up_data.attr_lv, 20);
    assert_eq!(outcome.user_up_data.attr_exp, 0);
    assert_eq!(
        (
            outcome.user_up_data.items[0].item_id,
            outcome.user_up_data.items[0].amount,
        ),
        (100_100_006, 0)
    );
    assert_eq!(player.roles[0].role_basic_info.as_ref().unwrap().exp, 0);
}

#[test]
fn role_level_up_refunds_cap_overflow_as_exp_materials() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role = player.roles[0].role_basic_info.as_mut().unwrap();
    let role_id = role.game_role_id;
    role.level = 19;
    role.exp = 800;
    player.add_item(100_300_002, 1, &tables).unwrap();
    player.add_item(100_100_003, 100_000, &tables).unwrap();

    let outcome = player
        .level_up_role(
            role_id,
            &[DcNetDataUseItem {
                item_id: 100_300_002,
                amount: 1,
                ..Default::default()
            }],
            &tables,
            1_787_523_760,
            config.server.zone_offset,
        )
        .unwrap();

    assert_eq!(
        (outcome.user_up_data.attr_lv, outcome.user_up_data.attr_exp),
        (20, 0)
    );
    assert_eq!(outcome.user_up_data.items.len(), 2);
    assert_eq!(
        (
            outcome.user_up_data.items[1].item_id,
            outcome.user_up_data.items[1].amount,
        ),
        (100_300_001, 4)
    );
}

#[test]
fn role_level_and_partner_change_match_captured_mutations() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = 110_100_013;
    let role = player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap();
    role.level = 6;
    role.exp = 420;
    if let Some(gold) = player
        .items
        .iter_mut()
        .find(|item| item.item_id == 100_100_003)
    {
        gold.amount = 251_100;
    } else {
        player.items.push(DcNetDataItem {
            item_id: 100_100_003,
            amount: 251_100,
            quality: 3,
            ..Default::default()
        });
    }
    player.items.push(DcNetDataItem {
        item_id: 100_300_001,
        amount: 11,
        quality: 3,
        ..Default::default()
    });

    let outcome = player
        .level_up_role(
            role_id,
            &[DcNetDataUseItem {
                item_id: 100_300_001,
                amount: 1,
                ..Default::default()
            }],
            &tables,
            1_787_523_760,
            config.server.zone_offset,
        )
        .unwrap();
    assert_eq!(
        (
            outcome.user_up_data.attr_lv,
            outcome.user_up_data.attr_exp,
            outcome.user_up_data.add_exp,
            outcome.user_up_data.is_lv_up,
        ),
        (8, 280, 1_000, true)
    );
    assert_eq!(
        outcome
            .remain
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_300_001, 10), (100_100_003, 249_000)]
    );
    assert_eq!(outcome.talents.len(), 15);

    let partner = DcNetDataPartner {
        id: 42_100_400_026,
        lv: 1,
        partner_id: 100_400_026,
        quality: 5,
        reson_lv: 1,
        skill_lv: 1,
        ..Default::default()
    };
    player.partners.push(partner);
    let changed = player.change_partner(partner.id, role_id).unwrap();
    assert_eq!(changed.partner, partner);
    assert_eq!(changed.role_info.user_partner_id, partner.id);
    assert!(changed.unset_role_info.is_none());

    let other_role_id = 110_100_006;
    let moved = player.change_partner(partner.id, other_role_id).unwrap();
    assert_eq!(moved.unset_role_info.unwrap().game_role_id, role_id);
    assert_eq!(moved.role_info.user_partner_id, partner.id);
    let partner2 = DcNetDataPartner {
        id: partner.id + 1,
        ..partner
    };
    player.partners.push(partner2);
    player.change_partner(partner2.id, role_id).unwrap();
    assert_eq!(player.lock_partner(partner.id, 1).unwrap().locked, 1);

    let swapped = player.swap_partners(partner.id, partner2.id).unwrap();
    assert_eq!(
        (
            swapped.role_info1.game_role_id,
            swapped.role_info1.user_partner_id
        ),
        (other_role_id, partner2.id)
    );
    assert_eq!(
        (
            swapped.role_info2.game_role_id,
            swapped.role_info2.user_partner_id
        ),
        (role_id, partner.id)
    );
    let unset = player.unset_partner(partner.id).unwrap();
    assert_eq!(unset.partner.id, partner.id);
    assert_eq!(
        (
            unset.role_info.game_role_id,
            unset.role_info.user_partner_id
        ),
        (role_id, 0)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .roles
            .iter()
            .find_map(|role| role
                .role_basic_info
                .as_ref()
                .filter(|role| role.game_role_id == other_role_id))
            .unwrap()
            .user_partner_id,
        partner2.id
    );
    assert_eq!(
        restored
            .partners
            .iter()
            .find(|candidate| candidate.id == partner.id)
            .unwrap()
            .locked,
        1
    );
}

#[test]
fn role_level_up_emits_captured_progress_updates() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    // The extracted table snapshot ends with battle pass 1003 on 2026-09-17.
    let now = 1_787_587_200;
    player.created_at = now - 86_400;
    let role_id = 110_100_013;
    let role = player
        .roles
        .iter_mut()
        .find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|role| role.game_role_id == role_id)
        })
        .unwrap();
    role.level = 19;
    role.exp = 0;
    player.daily_tasks.retain(|task| task.id != 3);
    player.daily_tasks.push(DcNetDataTaskCycle {
        id: 3,
        total: 1,
        ..Default::default()
    });
    player.add_item(100_300_001, 16, &tables).unwrap();
    if let Some(gold) = player
        .items
        .iter_mut()
        .find(|item| item.item_id == 100_100_003)
    {
        gold.amount = 1_000_000;
    } else {
        player.items.push(DcNetDataItem {
            item_id: 100_100_003,
            amount: 1_000_000,
            quality: 3,
            ..Default::default()
        });
    }

    let outcome = player
        .level_up_role(
            role_id,
            &[DcNetDataUseItem {
                item_id: 100_300_001,
                amount: 16,
                ..Default::default()
            }],
            &tables,
            now,
            config.server.zone_offset,
        )
        .unwrap();

    assert_eq!(player.item_spent.get(&100_300_001), Some(&16));
    assert_eq!(player.item_spent.get(&100_100_003), Some(&33_600));
    assert!(
        outcome.battle_pass_updates.iter().any(|task| {
            task.id == 10_032_060 && (task.progress, task.total) == (33_600, 500_000)
        })
    );

    assert!(outcome.achievement_updates.iter().any(|achievement| {
        achievement.id == 100_401 && achievement.progress == 20 && achievement.total == 20
    }));
    assert!(
        outcome
            .seven_day_updates
            .iter()
            .any(|activity| { activity.id == 7 && activity.progress == 1 && activity.total == 1 })
    );
    assert_eq!(
        outcome.daily_updates,
        [DcNetDataTaskCycle {
            id: 3,
            progress: 1,
            total: 1,
            ..Default::default()
        }]
    );
}
