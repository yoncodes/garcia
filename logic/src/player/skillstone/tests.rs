use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    (tables, config)
}

#[test]
fn skillstone_lifecycle_syncs_roles_rewards_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = tables.default_main_maid();
    let stone_ids = tables
        .skill_stones
        .rows
        .iter()
        .filter(|stone| stone.under_maid == role_id)
        .map(|stone| stone.id)
        .take(2)
        .collect::<Vec<_>>();
    assert_eq!(stone_ids.len(), 2);
    let first = player.grant_skillstone(stone_ids[0], &tables).unwrap();
    let second = player.grant_skillstone(stone_ids[1], &tables).unwrap();

    player
        .set_skillstone(role_id, first.user_stone_id, 1, &tables)
        .unwrap();
    let (equipped, dropped) = player
        .set_skillstone(role_id, second.user_stone_id, 1, &tables)
        .unwrap();
    assert_eq!(equipped.equiped_role, role_id);
    assert_eq!(dropped, vec![first]);
    assert_eq!(
        player
            .roles
            .iter()
            .find(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|info| info.game_role_id == role_id)
            })
            .unwrap()
            .skillstones,
        vec![equipped]
    );
    assert!(
        player
            .lock_skillstone(second.user_stone_id, true)
            .unwrap()
            .locked
    );
    player
        .set_skillstone(role_id, first.user_stone_id, 2, &tables)
        .unwrap();
    let (first_after, second_after) = player
        .swap_skillstones(first.user_stone_id, second.user_stone_id, &tables)
        .unwrap();
    assert_eq!((first_after.pos, second_after.pos), (1, 2));
    assert_eq!(player.unset_skillstone(role_id, 1).unwrap().len(), 1);

    let mut rewards = DcNetDataTakeRewardRes::default();
    player.add_reward(stone_ids[0], 1, &tables, 10, &mut rewards);
    assert_eq!(rewards.stone_rewards.len(), 1);
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.skillstones, player.skillstones);
    assert_eq!(restored.roles, player.roles);
}

#[test]
fn skillstone_role_validation_is_atomic() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = tables.default_main_maid();
    let config = tables
        .skill_stones
        .rows
        .iter()
        .find(|stone| stone.under_maid != role_id)
        .unwrap();
    let stone = player.grant_skillstone(config.id, &tables).unwrap();
    let before = player.skillstones.clone();

    assert_eq!(
        player.set_skillstone(role_id, stone.user_stone_id, 1, &tables),
        Err(SkillStoneError::WrongRole {
            stone_id: stone.user_stone_id,
            expected: config.under_maid,
            actual: role_id,
        })
    );
    assert_eq!(player.skillstones, before);
}
