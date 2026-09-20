use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn naming_uses_the_client_character_rules_and_table_cost() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    assert!(player.nickname.is_empty());

    assert!(
        player
            .rename(1, "Garcia42".into(), 1, &tables)
            .unwrap()
            .is_empty()
    );
    assert_eq!(player.gender(&tables), 1);
    let selected = tables.cultivation_constants.default_main_maid_sex[1];
    assert!(player.roles.iter().any(|role| {
        role.role_basic_info
            .as_ref()
            .is_some_and(|role| role.game_role_id == selected && role.cur_mc)
    }));
    assert!(
        player
            .formations
            .iter()
            .flat_map(|formation| &formation.poss)
            .any(|position| position.game_role_id == selected)
    );
    player.add_item(100_100_002, 150, &tables).unwrap();
    let remains = player.rename(2, "加西亚".into(), 0, &tables).unwrap();
    assert_eq!(player.nickname, "加西亚");
    assert_eq!((remains[0].item_id, remains[0].amount), (100_100_002, 50));

    let before = player.to_record();
    assert_eq!(
        player.rename(2, "invalid name".into(), 0, &tables),
        Err(NamingError::InvalidName)
    );
    assert_eq!(player.to_record(), before);

    let mut legacy = player.to_record();
    legacy.nickname = format!("Player{}", legacy.uid);
    assert!(
        Player::from_record(legacy, &tables, &config.account_defaults)
            .nickname
            .is_empty()
    );
}

#[test]
fn profile_selection_requires_an_owned_profile() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let avatar = player.profile_avatars[0].id;

    assert_eq!(player.select_profile(1, avatar), Ok(false));
    assert_eq!(
        player.select_profile(1, i32::MAX),
        Err(ProfileSelectionError::Locked {
            profile_type: 1,
            id: i32::MAX,
        })
    );
}

#[test]
fn bulk_profile_unlocks_add_only_missing_frames_and_titles() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let frames = player.grant_all_profile_frames(&tables, 123);
    let titles = player.grant_all_profile_titles(&tables, 123);
    assert_eq!(
        player.profile_frames.len(),
        tables.profile_frames.rows.len()
    );
    assert_eq!(
        player.profile_titles.len(),
        tables.profile_titles.rows.len()
    );
    assert!(!frames.profileframe_rewards.is_empty());
    assert!(!titles.profiletitle_rewards.is_empty());
    assert!(
        player
            .grant_all_profile_frames(&tables, 124)
            .profileframe_rewards
            .is_empty()
    );
    assert!(
        player
            .grant_all_profile_titles(&tables, 124)
            .profiletitle_rewards
            .is_empty()
    );
}
