use super::*;

#[test]
fn role_customization_and_main_character_switch_persist() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let current_id = tables.cultivation_constants.default_main_maid_sex[0];
    let target_id = tables.cultivation_constants.default_main_maid_sex[1];

    player
        .change_role_appearance(protocol::pbcommon::DcNetDataRolesChangeAppear {
            game_role_id: current_id,
            key: 1,
        })
        .unwrap();
    player.change_role_element(current_id, 2, false).unwrap();
    player.change_role_element(current_id, 4, true).unwrap();
    player.change_banner_girl(current_id).unwrap();

    player.equips.push(DcNetDataEquip {
        user_equip_id: 7,
        equiped_role: current_id,
        pos: 1,
        ..Default::default()
    });
    player.partners.push(DcNetDataPartner {
        id: 9,
        ..Default::default()
    });
    let current = player
        .roles
        .iter_mut()
        .find(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|info| info.game_role_id == current_id)
        })
        .unwrap();
    current.role_basic_info.as_mut().unwrap().user_partner_id = 9;
    player
        .skillstones
        .push(protocol::pbcommon::DcNetDataSkillStone {
            user_stone_id: 11,
            equiped_role: current_id,
            ..Default::default()
        });
    player.sync_role_skillstones();

    let outcome = player.switch_main_character(target_id, &tables).unwrap();
    assert!(!outcome.previous.role_basic_info.unwrap().cur_mc);
    assert!(outcome.current.role_basic_info.unwrap().cur_mc);
    assert_eq!(outcome.dropped_equips[0].equiped_role, 0);
    assert_eq!(outcome.dropped_partner.unwrap().id, 9);
    assert_eq!(outcome.dropped_skillstones[0].equiped_role, 0);
    assert!(player.formations.iter().all(|formation| {
        formation
            .poss
            .iter()
            .all(|pos| pos.game_role_id != current_id)
    }));
    assert!(player.role_profiles().contains(&target_id));

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.banner_girl, current_id);
    assert_eq!(restored.roles, player.roles);
    assert_eq!(restored.formations, player.formations);
    assert_eq!(restored.equips, player.equips);
}

#[test]
fn role_customization_rejects_invalid_values() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = tables.default_main_maid();

    assert_eq!(
        player.change_role_element(role_id, 0, false),
        Err(RoleMutationError::InvalidElement(0))
    );
    assert_eq!(
        player.change_role_appearance(protocol::pbcommon::DcNetDataRolesChangeAppear {
            game_role_id: role_id,
            key: 9,
        }),
        Err(RoleMutationError::InvalidAppearKey(9))
    );
}
fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn unlocked_role_skin_changes_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let skin = tables.maid_skins.get(110_400_801).unwrap();
    player
        .roles
        .push(gacha_role(player.uid, skin.maid_id, &tables).unwrap());

    assert_eq!(
        player.set_role_skin(skin.maid_id, skin.id, &tables),
        Err(RoleMutationError::LockedSkin {
            role_id: skin.maid_id,
            skin_id: skin.id,
        })
    );
    let mut reward = DcNetDataTakeRewardRes::default();
    player.add_reward(skin.id, 1, &tables, 0, &mut reward);
    assert_eq!(reward.skin_rewards, [skin.id]);
    player
        .set_role_skin(skin.maid_id, skin.id, &tables)
        .unwrap();

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.skins, [skin.id]);
    assert_eq!(
        restored
            .roles
            .iter()
            .filter_map(|role| role.role_basic_info.as_ref())
            .find(|role| role.game_role_id == skin.maid_id)
            .unwrap()
            .skin_id,
        skin.id
    );
}
