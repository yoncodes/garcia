use protocol::{
    cs::{
        DcNetWorkingParamExchangeInfo, DcNetWorkingParamExchangePhy,
        DcNetWorkingParamExchangeSbs2hhs, DcNetWorkingResExchangeInfo, DcNetWorkingResExchangePhy,
        DcNetWorkingResExchangeSbs2hhs,
    },
    prost::Message,
};

use crate::{
    logic::HeatExchangeOutcome,
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamExchangeInfo::decode(request.payload.as_slice())?;
    let count = ctx
        .player()?
        .heat_exchange_count(common::time::ServerTime::now_seconds_i32());
    ctx.send_reply(
        &request,
        DcNetWorkingResExchangeInfo {
            exchange_times_today: count,
        },
    )
}

pub async fn on_heat(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamExchangePhy::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let HeatExchangeOutcome {
        cost_remain,
        heat_remain,
        exchange_count,
    } = ctx
        .update_player()?
        .exchange_heat(
            request.amount,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidHeatExchange(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResExchangePhy {
            cost_remain: Some(cost_remain),
            phy_remain: Some(heat_remain),
            exchange_times_today: exchange_count,
        },
    )
}

pub async fn on_diamond_to_stamp(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamExchangeSbs2hhs::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let item = ctx
        .update_player()?
        .exchange_diamonds_for_stamps(request.amount, tables)
        .map_err(|error| NetworkError::InvalidExchange(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResExchangeSbs2hhs { item })
}
