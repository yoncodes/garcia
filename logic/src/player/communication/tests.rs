use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn red_dots_are_table_driven_and_checked_ids_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.record_user_guide("guide_prologue_move".into());
    let now = 1_787_522_879;
    let zone_offset = config.server.zone_offset;

    let mut ids: Vec<_> = player
        .red_dots(&tables, now, zone_offset)
        .into_iter()
        .map(|red_dot| red_dot.id)
        .collect();
    ids.sort();
    let mut captured = vec![
        "25_1006",
        "25_1011",
        "25_5501",
        "26_2",
        "26_7",
        "26_8",
        "28_101000006",
        "28_101000013",
        "28_101000016",
        "29_101100001",
        "30_101300001",
        "31_101200001",
    ];
    captured.sort();
    assert_eq!(ids, captured);

    let checked_id = "28_101000016".to_owned();
    let (checked, changed) =
        player.check_red_dots(std::slice::from_ref(&checked_id), &tables, now, zone_offset);
    assert!(changed);
    assert_eq!(checked.len(), 1);
    assert_eq!(checked[0].arg, "101000016");
    assert_eq!(checked[0].func_id, Reddot::Avatar as i32);
    assert!(checked[0].is_checked);
    assert!(
        player
            .red_dots(&tables, now, zone_offset)
            .iter()
            .all(|red_dot| red_dot.id != checked_id)
    );

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.checked_red_dots, [checked_id]);
}

#[test]
fn reward_item_red_dots_only_include_storage_packages() {
    let (tables, config) = fixture();
    let player = Player::new(42, &tables, &config.account_defaults);
    let reward = DcNetDataTakeRewardRes {
        items: vec![
            DcNetDataItem {
                item_id: 100_302_002,
                amount: 1,
                ..Default::default()
            },
            DcNetDataItem {
                item_id: 100_300_001,
                amount: 1,
                ..Default::default()
            },
            DcNetDataItem {
                item_id: 100_100_003,
                amount: 1,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let red_dots = player.reward_item_red_dots(&reward, &tables);
    assert_eq!(red_dots.len(), 1);
    assert_eq!(red_dots[0].id, "1_100302002");
    assert_eq!(red_dots[0].func_id, Reddot::Item as i32);

    let mut player = player;
    let id = red_dots[0].id.clone();
    let (checked, changed) = player.check_red_dots(std::slice::from_ref(&id), &tables, 0, 0);
    assert!(changed);
    assert_eq!(checked.len(), 1);
    assert!(checked[0].is_checked);
    assert_eq!(checked[0].arg, "100302002");
}

#[test]
fn unlocked_features_remain_in_the_red_dot_list_until_checked() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.feats.push(DcNetDataFeat {
        id: 115,
        status: FeatStatus::FeatUnlocked as i32,
    });

    let now = 1_787_522_879;
    let zone_offset = config.server.zone_offset;
    assert!(
        player
            .red_dots(&tables, now, zone_offset)
            .iter()
            .any(|red_dot| red_dot.id == "19_115")
    );

    let id = "19_115".to_owned();
    let (checked, changed) =
        player.check_red_dots(std::slice::from_ref(&id), &tables, now, zone_offset);
    assert!(changed);
    assert_eq!(checked[0].func_id, Reddot::Feat as i32);
    assert!(
        player
            .red_dots(&tables, now, zone_offset)
            .iter()
            .all(|red_dot| red_dot.id != id)
    );
}

#[test]
fn unlocked_albums_remain_in_the_red_dot_list_until_checked() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.albums.push(DcNetDataAlbums {
        id: 2013,
        status: AlbumStatus::Unlocked as i32,
        sort: 0,
    });

    assert!(
        player
            .red_dots(&tables, 123, 0)
            .iter()
            .any(|red_dot| red_dot.id == "27_2013")
    );

    let (_, changed) = player.check_red_dots(&["27_2013".into()], &tables, 123, 0);
    assert!(changed);
    assert!(
        !player
            .red_dots(&tables, 123, 0)
            .iter()
            .any(|red_dot| red_dot.id == "27_2013")
    );
}

#[test]
fn level_gated_lore_archives_use_their_table_conditions() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 1;
    let previous = player.unlocked_archive_ids(&tables, 123);
    assert!(!previous.contains(&3001));

    player.level = 2;
    let updates = player.newly_unlocked_archives_since(&previous, &tables, 456);
    assert!(updates.iter().any(|archive| archive.id == 3001));
    assert_eq!(
        player
            .archive_infos(&tables, 456)
            .iter()
            .find(|archive| archive.id == 3001)
            .map(|archive| archive.in_time),
        Some(456)
    );
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(
        restored
            .archive_infos(&tables, 789)
            .iter()
            .find(|archive| archive.id == 3001)
            .map(|archive| archive.in_time),
        Some(456)
    );
    assert!(
        player
            .red_dots(&tables, 123, 0)
            .iter()
            .any(|red_dot| red_dot.id == "25_3001")
    );
}

#[test]
fn mail_read_take_and_delete_match_the_client_contract() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = 1_787_522_879;
    let mail = |email_id, gifts, expires_at| MailState {
        mail: DcNetDataEmail {
            email_id,
            sender: 2,
            title: format!("Mail {email_id}"),
            gift_list: gifts,
            ..Default::default()
        },
        expires_at,
    };
    player.mails = vec![
        mail(
            1,
            vec![
                protocol::pbcommon::DcNetDataReward {
                    reward: 100_100_002,
                    amount: 300,
                    ..Default::default()
                },
                protocol::pbcommon::DcNetDataReward {
                    reward: 100_100_003,
                    amount: 30_000,
                    ..Default::default()
                },
            ],
            now + 100,
        ),
        mail(
            2,
            vec![protocol::pbcommon::DcNetDataReward {
                reward: 100_100_002,
                amount: 500,
                ..Default::default()
            }],
            0,
        ),
        mail(3, Vec::new(), 0),
    ];

    assert_eq!(player.mails(now)[0].expire, 100);
    let (read, changed) = player.read_mails(&[3], now);
    assert_eq!(read, [3]);
    assert!(changed);

    let before_diamonds = player
        .items
        .iter()
        .find(|item| item.item_id == 100_100_002)
        .map_or(0, |item| item.amount);
    let (rewards, taken, changed) = player.claim_mail_attachments(&[1, 2, 99], &tables, now);
    assert_eq!(taken, [1, 2]);
    assert!(changed);
    assert_eq!(rewards.items.len(), 2);
    assert_eq!(
        rewards
            .items
            .iter()
            .find(|item| item.item_id == 100_100_002)
            .unwrap()
            .amount,
        before_diamonds + 800
    );
    assert!(
        player.mails[..2]
            .iter()
            .all(|state| state.mail.is_read == 1 && state.mail.taken == 1)
    );
    assert!(
        player
            .claim_mail_attachments(&[1, 2], &tables, now)
            .1
            .is_empty()
    );

    let mut restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.mails, player.mails);
    assert_eq!(restored.delete_mails(&[1, 2, 3, 99]), [1, 2, 3]);
    assert!(restored.mails.is_empty());
}
