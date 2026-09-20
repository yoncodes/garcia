use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn boss_rush_keeps_best_runs_claims_thresholds_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let event = tables.boss_rushes.get(1005).unwrap();
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;
    let before = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_002)
        .map_or(0, |item| item.amount);

    let (score, _) = player
        .settle_boss_rush(event, 180_800_002, 40_000, vec![role_id])
        .unwrap();
    assert_eq!(score, 40_000);
    let (score, _) = player
        .settle_boss_rush(event, 180_800_002, 20_000, vec![role_id])
        .unwrap();
    assert_eq!(score, 40_000);
    let (score, _) = player
        .settle_boss_rush(event, 180_800_001, 200_000, vec![role_id])
        .unwrap();
    assert_eq!(score, 240_000);

    let (_, claimed) = player
        .claim_boss_rush_rewards(1005, 0, &tables, 123)
        .unwrap();
    assert_eq!(claimed, [41, 42]);
    assert_eq!(
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_100_002)
            .unwrap()
            .amount,
        before + 50
    );
    assert_eq!(
        player.claim_boss_rush_rewards(1005, 43, &tables, 124),
        Err(BossRushError::RewardNotReached(43))
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.boss_rush, player.boss_rush);
}

#[test]
fn ranking_reward_is_mailed_once_and_persists() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let event = tables.boss_rushes.get(1005).unwrap();
    player
        .settle_boss_rush(event, event.port_id[0], 1, Vec::new())
        .unwrap();

    let mail = player
        .deliver_boss_rush_ranking_reward(1005, 1, 1, &tables, 123)
        .unwrap();
    assert_eq!(mail.sys_mail_id, 5);
    assert_eq!(mail.paramter, "1");
    assert_eq!(mail.gift_list[0].reward, 100_100_002);
    assert_eq!(mail.gift_list[0].amount, 1_000);
    assert_eq!(mail.gift_list.len(), 5);
    assert!(
        player
            .deliver_boss_rush_ranking_reward(1005, 1, 1, &tables, 124)
            .is_none()
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.boss_rush.ranking_rewards, [1005]);
    assert_eq!(restored.mails.len(), 1);
}

#[test]
fn ranking_mail_claim_reports_owned_title_attachments() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.grant_all_profile_titles(&tables, 100);
    let event = tables.boss_rushes.get(1001).unwrap();
    player
        .settle_boss_rush(event, event.port_id[0], 1, Vec::new())
        .unwrap();
    let mail = player
        .deliver_boss_rush_ranking_reward(1001, 1, 1, &tables, 123)
        .unwrap();

    let (rewards, claimed, changed) = player.claim_mail_attachments(&[mail.email_id], &tables, 124);
    assert!(changed);
    assert_eq!(claimed, [mail.email_id]);
    assert_eq!(rewards.items.len(), 1);
    assert_eq!(rewards.profiletitle_rewards.len(), 4);
}
