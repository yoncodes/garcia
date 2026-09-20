use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn starter_state_comes_from_extracted_tables_and_mutates() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert_eq!(player.items.len(), 2);
    assert_eq!(player.roles.len(), 3);
    assert_eq!(
        player
            .formations
            .iter()
            .map(|formation| formation.formation_id)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );
    assert_eq!(
        player
            .formations
            .iter()
            .find(|formation| formation.formation_id == player.cur_form)
            .unwrap()
            .poss
            .iter()
            .map(|position| position.game_role_id)
            .collect::<Vec<_>>(),
        [110_100_016, 110_100_013]
    );
    assert_eq!(player.locals[0].local, "nh_prologue_tp1");
    assert_eq!(player.dense_fogs, [4_010_661]);
    assert_eq!(player.interact_objs.len(), 25);
    assert!(tables.is_interaction_object("nh_field_a_Map_TP"));
    assert_eq!(player.challenges.len(), 24);
    assert_eq!(player.albums.len(), 1);
    assert!(player.city_guides.is_empty());
    assert!(
        player
            .record_city_guide("guide_prologue_move".into(), &tables)
            .unwrap()
    );
    assert!(
        !player
            .record_city_guide("guide_prologue_move".into(), &tables)
            .unwrap()
    );
    assert_eq!(player.city_guides, ["guide_prologue_move"]);
    assert_eq!(player.interact_objs.len(), 25);
}
