use protocol::{
    cs::{
        DcNetWorkingParamItemSelectReward, DcNetWorkingParamItemsConvert,
        DcNetWorkingParamItemsGet, DcNetWorkingParamItemsSynthesis, DcNetWorkingParamItemsUseItem,
        DcNetWorkingParamRebateItemUse, DcNetWorkingResItemSelectReward,
        DcNetWorkingResItemsConvert, DcNetWorkingResItemsGet, DcNetWorkingResItemsSynthesis,
        DcNetWorkingResItemsUseItem,
    },
    prost::Message,
};

use crate::{
    logic::{CraftError, CraftOutcome, InventoryError, ItemUseOutcome},
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamItemsGet::decode(packet.payload.as_slice())?;
    let items = match request.items {
        Some(filter) => ctx
            .player()?
            .items
            .iter()
            .filter(|item| {
                (filter.user_item_id == 0 || filter.user_item_id == item.user_item_id)
                    && (filter.item_id == 0 || filter.item_id == item.item_id)
            })
            .cloned()
            .collect(),
        None => ctx.player()?.items.clone(),
    };
    ctx.send_reply(&packet, DcNetWorkingResItemsGet { items })
}

pub async fn on_convert(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamItemsConvert::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let CraftOutcome {
        id,
        amount,
        remains,
        target,
    } = ctx
        .update_player()?
        .convert_item(request.id, request.amount, &request.costs, tables, now)
        .map_err(invalid_craft)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResItemsConvert {
            id,
            amount,
            remains,
            target: Some(target),
        },
    )
}

pub async fn on_synthesis(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamItemsSynthesis::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let CraftOutcome {
        id,
        amount,
        remains,
        target,
    } = ctx
        .update_player()?
        .synthesize_item(request.id, request.amount, tables, now)
        .map_err(invalid_craft)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResItemsSynthesis {
            id,
            amount,
            remains,
            target: Some(target),
        },
    )
}

pub async fn on_use(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamItemsUseItem::decode(packet.payload.as_slice())?;
    let item = request
        .items
        .ok_or_else(|| NetworkError::InvalidInventory("missing item".into()))?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let ItemUseOutcome { rewards, remain } = ctx
        .update_player()?
        .use_item(item, tables, common::time::ServerTime::now_seconds_i32())
        .map_err(invalid_item)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResItemsUseItem {
            items_equips: Some(rewards),
            remains: Some(remain),
        },
    )
}

pub async fn on_select_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamItemSelectReward::decode(packet.payload.as_slice())?;
    let selections = request
        .selections
        .iter()
        .map(|selection| (selection.rid, selection.amount))
        .collect::<Vec<_>>();
    let state = ctx.state.clone();
    let tables = &state.tables;
    let ItemUseOutcome { rewards, remain } = ctx
        .update_player()?
        .select_package_reward(
            request.item_id,
            &selections,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_item)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResItemSelectReward {
            remain: Some(remain),
            reward: Some(rewards),
        },
    )
}

pub async fn on_rebate_item_use(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRebateItemUse::decode(packet.payload.as_slice())?;
    let item_type = ctx
        .state
        .tables
        .items
        .get(request.item_id)
        .map(|item| item.item_type)
        .ok_or_else(|| {
            NetworkError::InvalidInventory(format!("unknown item {}", request.item_id))
        })?;
    let reason = match item_type {
        901 => "monthly-card activation is not recovered",
        902 => "battle-pass activation is not recovered",
        _ => "item is not a rebate item",
    };
    Err(NetworkError::InvalidInventory(format!(
        "item {} (type {}): {}",
        request.item_id, item_type, reason
    )))
}

fn invalid_craft(error: CraftError) -> NetworkError {
    NetworkError::InvalidInventory(error.to_string())
}

fn invalid_item(error: InventoryError) -> NetworkError {
    NetworkError::InvalidInventory(error.to_string())
}
