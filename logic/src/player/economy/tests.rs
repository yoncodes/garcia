use super::*;

#[test]
fn recharge_uses_table_rewards_and_persists_monthly_card_time() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let balance = |player: &Player| {
        player
            .items
            .iter()
            .find(|item| item.item_id == 100_100_001)
            .map_or(0, |item| item.amount)
    };

    let before = balance(&player);
    player.buy_recharge(101, &tables, 1_000).unwrap();
    assert_eq!(balance(&player) - before, 120);
    let before = balance(&player);
    player.buy_recharge(101, &tables, 1_001).unwrap();
    assert_eq!(balance(&player) - before, 60);

    let before = balance(&player);
    player.buy_recharge(201, &tables, 2_000).unwrap();
    assert_eq!(balance(&player) - before, 300);
    let state = player
        .recharge_purchases
        .iter()
        .find(|purchase| purchase.recharge_id == 201)
        .unwrap();
    assert_eq!(state.expires_at, 2_000 + 30 * 86_400);

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.recharge_purchases, player.recharge_purchases);
}

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn reward_boxes_use_fixed_and_breakable_drop_tables_once() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let fixed = player
        .claim_reward_box("nh_field_a_RewardBox_05", &tables, 123)
        .unwrap();
    assert_eq!(
        (fixed.items[0].item_id, fixed.items[0].amount),
        (100_100_002, 30)
    );
    let breakable = player
        .claim_reward_box("nh_field_a_break_9", &tables, 124)
        .unwrap();
    assert_eq!(
        (breakable.items[0].item_id, breakable.items[0].amount),
        (100_100_003, 100)
    );
    assert_eq!(
        player.reward_boxes(),
        [
            DcNetDataTBoxInfo {
                id: "nh_field_a_RewardBox_05".into(),
                status: 1,
            },
            DcNetDataTBoxInfo {
                id: "nh_field_a_break_9".into(),
                status: 1,
            }
        ]
    );
    assert_eq!(
        player.claim_reward_box("nh_field_a_RewardBox_05", &tables, 125),
        Err(RewardBoxError::AlreadyClaimed(
            "nh_field_a_RewardBox_05".into()
        ))
    );
}

#[test]
fn real_money_mall_goods_are_granted_without_an_inventory_cost() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    let first = tables
        .mall_goods
        .iter()
        .find(|goods| goods.id == 70_014_116)
        .unwrap();
    let first_ticket_before = item_balance(&player, 100_100_025);
    let first_crystal_before = item_balance(&player, 100_100_002);
    let partner_before = player.partners.len();
    let first_outcome = player
        .buy_mall_goods(first.id, 1, &tables, 123, config.server.zone_offset)
        .unwrap();

    assert!(first_outcome.remains.is_empty());
    assert_eq!(first_outcome.goods.bought, 1);
    assert_eq!(player.partners.len() - partner_before, 1);
    assert_eq!(first_outcome.reward.partner_rewards.len(), 1);
    assert_eq!(
        first_outcome.reward.partner_rewards[0]
            .partner
            .as_ref()
            .unwrap()
            .partner_id,
        100_463_900
    );
    assert_eq!(item_balance(&player, 100_100_025) - first_ticket_before, 1);
    assert_eq!(
        item_balance(&player, 100_100_002) - first_crystal_before,
        80
    );

    let second = tables
        .mall_goods
        .iter()
        .find(|goods| goods.id == 70_014_117)
        .unwrap();
    let second_ticket_before = item_balance(&player, 100_100_025);
    let second_crystal_before = item_balance(&player, 100_100_002);
    let material_before = item_balance(&player, 100_300_000);
    let second_outcome = player
        .buy_mall_goods(second.id, 1, &tables, 124, config.server.zone_offset)
        .unwrap();
    assert!(second_outcome.remains.is_empty());
    assert_eq!(item_balance(&player, 100_100_025) - second_ticket_before, 3);
    assert_eq!(
        item_balance(&player, 100_100_002) - second_crystal_before,
        320
    );
    assert_eq!(item_balance(&player, 100_300_000) - material_before, 2);
    assert_eq!(
        reward_item_amount(&second_outcome.reward, 100_100_002),
        item_balance(&player, 100_100_002)
    );

    assert_eq!(player.mall_bought(first.id, 0), 1);
    assert_eq!(player.mall_bought(second.id, 0), 1);
    assert!(matches!(
        player.buy_mall_goods(first.id, 1, &tables, 125, config.server.zone_offset),
        Err(MallPurchaseError::BuyLimit { id, limit })
            if id == first.id && limit == first.buy_limit
    ));
}

#[test]
fn purchases_cannot_bypass_inactive_table_windows() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let now = i32::MAX;

    assert_eq!(
        player.buy_shop_goods(10_030_104, 1, &tables, now, config.server.zone_offset),
        Err(ShopPurchaseError::Inactive(10_030_104))
    );
    assert!(matches!(
        player.buy_mall_goods(10_030_101, 1, &tables, now, config.server.zone_offset),
        Err(MallPurchaseError::Inactive(10_030_101))
    ));
}

#[test]
fn banner_override_reopens_its_exchange_goods_until_override_expiry() {
    let (tables, config) = fixture();
    let now = 1_787_522_877;
    let expires_at = now + 30 * 86_400;
    let goods_id = 10_030_101;
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.add_item(100_100_022, 80, &tables).unwrap();

    assert!(matches!(
        player.buy_mall_goods(goods_id, 1, &tables, now, config.server.zone_offset),
        Err(MallPurchaseError::Inactive(id)) if id == goods_id
    ));

    let outcome = player
        .buy_mall_goods_with_override(
            goods_id,
            1,
            &tables,
            now,
            config.server.zone_offset,
            Some(expires_at),
        )
        .unwrap();

    assert_eq!(
        (
            outcome.goods.id,
            outcome.goods.cd_time,
            outcome.goods.bought
        ),
        (goods_id, 30 * 86_400, 1)
    );
    assert_eq!(player.mall_bought(goods_id, expires_at), 1);
}

fn reward_item_amount(reward: &DcNetDataTakeRewardRes, item_id: i32) -> i32 {
    reward
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .map_or(0, |item| item.amount)
}

fn item_balance(player: &Player, item_id: i32) -> i32 {
    player
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .map_or(0, |item| item.amount)
}
