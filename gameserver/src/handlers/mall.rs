use protocol::cs::{
    DcNetWorkingParamMallBuy, DcNetWorkingParamMallList, DcNetWorkingParamRechargeList,
    DcNetWorkingParamShopBuy, DcNetWorkingParamShopGetGoods, DcNetWorkingParamShopGetInfo,
    DcNetWorkingParamShopSell, DcNetWorkingParamShopSetShipTags,
    DcNetWorkingParamShopShipTagRefresh, DcNetWorkingResMallBuy, DcNetWorkingResMallList,
    DcNetWorkingResRechargeList, DcNetWorkingResShopBuy, DcNetWorkingResShopGetGoods,
    DcNetWorkingResShopGetInfo, DcNetWorkingResShopSell,
};
use protocol::{
    pbcommon::{
        DcNetDataMall, DcNetDataMallGoods, DcNetDataMonthlyCard, DcNetDataRecharge, DcNetDataShop,
        DcNetDataShopGoods,
    },
    prost::Message,
};

use crate::{
    logic::{
        DisassemblyOutcome, ShopPurchaseError, ShopPurchaseOutcome, active_table_window,
        mall_goods_runtime_with_override, mall_group_runtime_with_override, shop_goods_runtime,
        shop_runtime,
    },
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_shop_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamShopGetInfo::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    ctx.send_reply(
        &request,
        DcNetWorkingResShopGetInfo {
            shops: shop_info(&ctx.state.tables, now, common::config().server.zone_offset),
            // The captured live response contains shops only. Ship-tag state is
            // supplied by a separate, currently unrecovered server flow.
            shopship: None,
        },
    )
}

pub async fn on_set_ship_tags(
    _ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamShopSetShipTags::decode(request.payload.as_slice())?;
    Err(NetworkError::InvalidShipTags(
        "ship-tag offered choices and response state are not recovered".into(),
    ))
}

pub async fn on_ship_tag_refresh(
    _ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamShopShipTagRefresh::decode(request.payload.as_slice())?;
    Err(NetworkError::InvalidShipTags(
        "ship-tag generation is not recovered".into(),
    ))
}

pub async fn on_shop_goods(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamShopGetGoods::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let state = ctx.state.clone();
    let tables = &state.tables;
    let zone_offset = common::config().server.zone_offset;
    let mut goods_list = shop_goods(tables, now, zone_offset);
    let player = ctx.player()?;
    for goods in &mut goods_list {
        if let Some((_, period)) = shop_goods_runtime(tables, goods.goods_id, now, zone_offset) {
            goods.bought = player.mall_bought(goods.goods_id, period);
        }
    }
    ctx.send_reply(&request, DcNetWorkingResShopGetGoods { goods_list })
}

pub async fn on_shop_buy(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    let param = DcNetWorkingParamShopBuy::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let state = ctx.state.clone();
    let tables = &state.tables;
    let zone_offset = common::config().server.zone_offset;
    let mut goods = shop_goods_row(tables, param.goods_id, now, zone_offset).ok_or_else(|| {
        NetworkError::InvalidMallPurchase(format!("shop goods {} is not active", param.goods_id))
    })?;
    let ShopPurchaseOutcome {
        reward,
        remains,
        bought,
    } = ctx
        .update_player()?
        .buy_shop_goods(param.goods_id, param.num, tables, now, zone_offset)
        .map_err(invalid_shop_purchase)?;
    goods.bought = bought;
    ctx.send_reply(
        &request,
        DcNetWorkingResShopBuy {
            take_res: Some(reward),
            remain: remains,
            goods: Some(goods),
        },
    )
}

pub async fn on_shop_sell(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    let param = DcNetWorkingParamShopSell::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let DisassemblyOutcome { sold, reward } = ctx
        .update_player()?
        .disassemble(&param.sell_rows, tables)
        .map_err(|error| NetworkError::InvalidDisassembly(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResShopSell {
            sell_rows: sold,
            take_res: Some(reward),
        },
    )
}

pub async fn on_mall_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMallList::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let mall_overrides = active_gacha_mall_overrides(ctx, now).await?;
    let mut malls = mall_list(
        &ctx.state.tables,
        now,
        common::config().server.zone_offset,
        &mall_overrides,
    );
    let state = ctx.state.clone();
    let tables = &state.tables;
    let zone_offset = common::config().server.zone_offset;
    let player = ctx.player()?;
    for goods in malls.iter_mut().flat_map(|mall| &mut mall.goods) {
        let override_expires_at = mall_override_for_goods(tables, &mall_overrides, goods.id, now);
        if let Some((_, period)) = mall_goods_runtime_with_override(
            tables,
            goods.id,
            now,
            zone_offset,
            override_expires_at,
        ) {
            goods.bought = player.mall_bought(goods.id, period);
        }
    }
    ctx.send_reply(&request, DcNetWorkingResMallList { malls })
}

pub async fn on_mall_buy(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    let param = DcNetWorkingParamMallBuy::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let state = ctx.state.clone();
    let tables = &state.tables;
    if tables.recharges.get(param.id).is_some() {
        let outcome = ctx
            .update_player()?
            .buy_recharge(param.id, tables, now)
            .map_err(|error| NetworkError::InvalidMallPurchase(error.to_string()))?;
        return ctx.send_reply(
            &request,
            DcNetWorkingResMallBuy {
                res: Some(outcome.reward),
                remain: Vec::new(),
                goods: Some(DcNetDataMallGoods {
                    id: param.id,
                    cd_time: 0,
                    bought: outcome.purchase_count,
                }),
            },
        );
    }
    let zone_offset = common::config().server.zone_offset;
    let mall_overrides = active_gacha_mall_overrides(ctx, now).await?;
    let override_expires_at = mall_override_for_goods(tables, &mall_overrides, param.id, now);
    let outcome = ctx
        .update_player()?
        .buy_mall_goods_with_override(
            param.id,
            param.amount,
            tables,
            now,
            zone_offset,
            override_expires_at,
        )
        .map_err(|error| NetworkError::InvalidMallPurchase(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResMallBuy {
            res: Some(outcome.reward),
            remain: outcome.remains,
            goods: Some(outcome.goods),
        },
    )
}

pub async fn on_recharge_list(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamRechargeList::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let player = ctx.player()?;
    let list = tables
        .recharges
        .rows
        .iter()
        .map(|row| DcNetDataRecharge {
            id: row.id,
            price: row.price,
            goods: row
                .goods
                .as_ref()
                .map(|value| [(value.key, value.value)].into())
                .unwrap_or_default(),
            first_goods: row
                .first_goods
                .as_ref()
                .map(|value| [(value.key, value.value)].into())
                .unwrap_or_default(),
            other_goods: row
                .other_goods
                .as_ref()
                .map(|value| [(value.key, value.value)].into())
                .unwrap_or_default(),
            had_first_taken: player
                .recharge_purchases
                .iter()
                .any(|purchase| purchase.recharge_id == row.id && purchase.purchase_count > 0),
        })
        .collect();
    let now = common::time::ServerTime::now_seconds_i32();
    let monthly_card = tables
        .recharges
        .rows
        .iter()
        .find(|row| row.kind == 2)
        .and_then(|row| {
            player
                .recharge_purchases
                .iter()
                .find(|purchase| purchase.recharge_id == row.id)
        })
        .map_or_else(DcNetDataMonthlyCard::default, |purchase| {
            DcNetDataMonthlyCard {
                cd_sec: i64::from(purchase.expires_at.saturating_sub(now).max(0)),
                bought_time: i64::from(purchase.last_bought_at),
                expired_time: i64::from(purchase.expires_at),
            }
        });
    ctx.send_reply(
        &request,
        DcNetWorkingResRechargeList {
            list,
            monthly_card: Some(monthly_card),
        },
    )
}

fn shop_info(tables: &configs::GameTables, now: i32, zone_offset: i32) -> Vec<DcNetDataShop> {
    tables
        .shops
        .rows
        .iter()
        .filter_map(|shop| {
            let remain_sec = shop_runtime(tables, shop.id, now, zone_offset)?;
            Some(DcNetDataShop {
                id: shop.id,
                remain_sec,
                goods_num: shop.item_num,
                shop_type: shop.shop_type,
            })
        })
        .collect()
}

fn shop_goods(tables: &configs::GameTables, now: i32, zone_offset: i32) -> Vec<DcNetDataShopGoods> {
    tables
        .free_shop_goods
        .iter()
        .filter_map(|goods| shop_goods_row(tables, goods.id, now, zone_offset))
        .collect()
}

fn shop_goods_row(
    tables: &configs::GameTables,
    goods_id: i32,
    now: i32,
    zone_offset: i32,
) -> Option<DcNetDataShopGoods> {
    let goods = tables
        .free_shop_goods
        .iter()
        .find(|goods| goods.id == goods_id)?;
    let (remain_sec, _) = shop_goods_runtime(tables, goods_id, now, zone_offset)?;
    Some(DcNetDataShopGoods {
        goods_id: goods.id,
        shop_id: goods.belong_shop,
        buy_limit: goods.buy_limit,
        bought: 0,
        goods: format!("{}:{}", goods.goods.key, goods.goods.value),
        price: format!("{}:{}", goods.price.key, goods.price.value),
        remain_sec,
    })
}

fn invalid_shop_purchase(error: ShopPurchaseError) -> NetworkError {
    NetworkError::InvalidMallPurchase(error.to_string())
}

fn mall_list(
    tables: &configs::GameTables,
    now: i32,
    zone_offset: i32,
    overrides: &[(i32, i32)],
) -> Vec<DcNetDataMall> {
    tables
        .malls
        .rows
        .iter()
        .filter(|mall| {
            active_table_window(&mall.mall_open, &mall.mall_close, now, zone_offset)
                || override_expiration(overrides, mall.id, now).is_some()
        })
        .map(|mall| {
            let override_expires_at = override_expiration(overrides, mall.id, now);
            let goods = tables
                .mall_goods_groups
                .iter()
                .filter(|group| group.mall_id == mall.id)
                .flat_map(|group| {
                    let Some((cd_time, _)) = mall_group_runtime_with_override(
                        tables,
                        group.goods_group_id,
                        now,
                        zone_offset,
                        override_expires_at,
                    ) else {
                        return Vec::new();
                    };
                    tables
                        .mall_goods
                        .iter()
                        .filter(move |goods| goods.goods_group_id == group.goods_group_id)
                        .map(move |goods| DcNetDataMallGoods {
                            id: goods.id,
                            cd_time,
                            bought: 0,
                        })
                        .collect()
                })
                .collect();
            DcNetDataMall {
                id: mall.id,
                cd_time: override_expires_at.map_or_else(
                    || {
                        common::time::table_time_utc(&mall.mall_close, zone_offset)
                            .map_or(0, |end| common::time::seconds_until(end, i64::from(now)))
                    },
                    |expires_at| expires_at - now,
                ),
                goods,
            }
        })
        .collect()
}

async fn active_gacha_mall_overrides(
    ctx: &HandlerContext,
    now: i32,
) -> NetworkResult<Vec<(i32, i32)>> {
    Ok(
        database::db::gacha_banner_overrides::active(&ctx.state.db, now)
            .await?
            .into_iter()
            .filter_map(|banner| {
                ctx.state
                    .tables
                    .gacha_pools
                    .get(banner.gacha_id)
                    .map(|pool| (pool.mall_id, banner.expires_at))
            })
            .collect(),
    )
}

fn override_expiration(overrides: &[(i32, i32)], mall_id: i32, now: i32) -> Option<i32> {
    overrides
        .iter()
        .find_map(|&(id, expires_at)| (id == mall_id && expires_at > now).then_some(expires_at))
}

fn mall_override_for_goods(
    tables: &configs::GameTables,
    overrides: &[(i32, i32)],
    goods_id: i32,
    now: i32,
) -> Option<i32> {
    let group_id = tables
        .mall_goods
        .iter()
        .find(|goods| goods.id == goods_id)?
        .goods_group_id;
    let mall_id = tables
        .mall_goods_groups
        .iter()
        .find(|group| group.goods_group_id == group_id)?
        .mall_id;
    override_expiration(overrides, mall_id, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Player;

    fn fixture() -> (configs::GameTables, common::config::ServerConfig) {
        let config = common::load_config().unwrap();
        let tables = configs::GameTables::load(&config.paths.game_tables).unwrap();
        (tables, config)
    }

    #[test]
    fn mall_and_shop_lists_match_captured_active_tables_and_timers() {
        let (tables, config) = fixture();
        let now = 1_787_522_877;
        let zone_offset = config.server.zone_offset;

        let shops = shop_info(&tables, now, zone_offset);
        assert_eq!(
            shops.iter().map(|shop| shop.id).collect::<Vec<_>>(),
            [1001, 100301]
        );
        assert_eq!(shops[1].remain_sec, 391_923);

        let goods = shop_goods(&tables, now, zone_offset);
        assert_eq!(goods.len(), 39);
        assert_eq!(
            goods
                .iter()
                .find(|goods| goods.goods_id == 100_109)
                .unwrap()
                .remain_sec,
            46_323
        );
        assert_eq!(
            goods
                .iter()
                .find(|goods| goods.goods_id == 100_118)
                .unwrap()
                .remain_sec,
            0
        );

        let malls = mall_list(&tables, now, zone_offset, &[]);
        assert_eq!(malls.len(), 12);
        let mall = |id| malls.iter().find(|mall| mall.id == id).unwrap();
        assert_eq!((mall(1007).cd_time, mall(1007).goods.len()), (420_723, 6));
        assert!(
            mall(1007)
                .goods
                .iter()
                .all(|goods| goods.cd_time == 305_522)
        );
        assert_eq!(
            mall(4002)
                .goods
                .iter()
                .find(|goods| goods.id == 40_020_104)
                .unwrap()
                .cd_time,
            737_523
        );

        let expires_at = now + 30 * 86_400;
        let malls = mall_list(&tables, now, zone_offset, &[(1003, expires_at)]);
        let historical = malls.iter().find(|mall| mall.id == 1003).unwrap();
        assert_eq!(historical.cd_time, 30 * 86_400);
        assert_eq!(historical.goods.len(), 6);
        assert!(
            historical
                .goods
                .iter()
                .all(|goods| goods.cd_time == 30 * 86_400)
        );
        assert_eq!(historical.goods[0].id, 10_030_101);
    }

    #[test]
    fn captured_free_mall_purchase_rewards_once_and_persists() {
        let (tables, config) = fixture();
        let now = 1_787_522_877;
        let mut player = Player::new(3_028_256_871, &tables, &config.account_defaults);
        let outcome = player
            .buy_mall_goods(70_010_701, 1, &tables, now, config.server.zone_offset)
            .unwrap();

        assert_eq!(
            (
                outcome.goods.id,
                outcome.goods.cd_time,
                outcome.goods.bought
            ),
            (70_010_701, 46_323, 1)
        );
        assert_eq!(outcome.reward.items.len(), 1);
        assert!(
            [
                100_100_002,
                100_302_002,
                100_302_005,
                100_302_001,
                100_300_001,
                100_300_004,
                100_300_007,
            ]
            .contains(&outcome.reward.items[0].item_id)
        );
        assert!(
            player
                .buy_mall_goods(70_010_701, 1, &tables, now, config.server.zone_offset)
                .is_err()
        );
        let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
        assert_eq!(restored.mall_purchases, player.mall_purchases);
    }

    #[test]
    fn shop_purchase_uses_table_price_limit_reward_and_persistence() {
        let (tables, config) = fixture();
        let now = 1_787_522_877;
        let goods = shop_goods_row(&tables, 10_030_104, now, config.server.zone_offset).unwrap();
        let (_, period) =
            shop_goods_runtime(&tables, 10_030_104, now, config.server.zone_offset).unwrap();
        assert_eq!(
            (goods.shop_id, goods.price.as_str()),
            (100_301, "100100303:800")
        );

        let mut player = Player::new(42, &tables, &config.account_defaults);
        player.items.push(protocol::pbcommon::DcNetDataItem {
            user_item_id: 100,
            item_id: 100_100_303,
            amount: 2_000,
            quality: 3,
            ..Default::default()
        });
        let bought = player
            .buy_shop_goods(10_030_104, 2, &tables, now, config.server.zone_offset)
            .unwrap();
        assert_eq!((bought.bought, bought.remains[0].amount), (2, 400));
        assert_eq!(
            (
                bought.reward.items[0].item_id,
                bought.reward.items[0].amount
            ),
            (100_100_025, 2)
        );

        let before = player.items.clone();
        assert!(
            player
                .buy_shop_goods(10_030_104, 2, &tables, now, config.server.zone_offset)
                .is_err()
        );
        assert_eq!(player.items, before);
        let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
        assert_eq!(restored.mall_bought(10_030_104, period), 2);

        player
            .items
            .iter_mut()
            .find(|item| item.item_id == 100_100_303)
            .unwrap()
            .amount = 1_500;
        let profile = player
            .buy_shop_goods(10_030_101, 1, &tables, now, config.server.zone_offset)
            .unwrap();
        assert_eq!(profile.reward.profileframe_rewards[0].id, 101_102_011);
        let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
        assert!(
            restored
                .profile_frames
                .iter()
                .any(|profile| profile.id == 101_102_011)
        );
    }
}
