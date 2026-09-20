use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn daily_activity_claims_every_reached_reward_tier() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    assert!(player.daily_tasks.iter().any(|task| task.id == 1));
    assert_eq!(
        player.claim_daily_activity(1, &tables).unwrap(),
        (1, 100, 0)
    );
    let outcome = player.claim_daily_rewards(&tables, 123).unwrap();

    assert_eq!(player.daily_reward_progress, 1);
    assert_eq!(
        outcome
            .rewards
            .items
            .iter()
            .map(|item| (item.item_id, item.amount))
            .collect::<Vec<_>>(),
        [(100_100_002, 10), (100_100_003, 11_000)]
    );
    assert_eq!(
        (outcome.level_up_data.attr_lv, outcome.level_up_data.add_exp),
        (3, 500)
    );
    assert_eq!(outcome.level_up_data.items[0].amount, 180);
    assert!(
        outcome
            .archive_updates
            .iter()
            .any(|archive| archive.id == 3001)
    );
    assert!(matches!(
        player.claim_daily_rewards(&tables, 124),
        Err(DailyTaskError::NoReward)
    ));
}
