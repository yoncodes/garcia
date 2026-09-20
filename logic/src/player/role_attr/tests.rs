use super::*;

#[test]
fn role_attributes_validate_and_persist() {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    let tables = GameTables::load(&versions).unwrap();
    let config = common::load_config().unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;

    player
        .save_role_attrs(vec![DcNetDataRoleAttrInfo {
            role_id,
            mp: 10,
            ep: 20,
            hp: 30,
        }])
        .unwrap();
    assert_eq!(
        player.save_role_attrs(vec![DcNetDataRoleAttrInfo {
            role_id: -1,
            ..Default::default()
        }]),
        Err(RoleAttrError::RoleNotOwned(-1))
    );
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.role_attrs, player.role_attrs);
}
