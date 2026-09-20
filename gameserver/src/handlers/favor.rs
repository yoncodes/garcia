use protocol::{
    cs::{
        DcNetWorkingParamFavorAdd, DcNetWorkingParamFavorList, DcNetWorkingParamFavorUseItem,
        DcNetWorkingResFavorAdd, DcNetWorkingResFavorList, DcNetWorkingResFavorUseItem,
    },
    prost::Message,
};

use crate::{
    logic::FavorError,
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_favor_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamFavorList::decode(packet.payload.as_slice())?;
    let list = ctx.player()?.favors.clone();
    ctx.send_reply(&packet, DcNetWorkingResFavorList { list })
}

pub async fn on_add(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFavorAdd::decode(packet.payload.as_slice())?;
    let character_id = character_id(request.character_id)?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let zone_offset = common::config().server.zone_offset;
    let (add_num, info) = ctx
        .update_player()?
        .touch_favor(character_id, tables, now, zone_offset)
        .map_err(invalid_favor)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFavorAdd {
            add_num,
            info: Some(info),
        },
    )
}

pub async fn on_use_item(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFavorUseItem::decode(packet.payload.as_slice())?;
    let character_id = character_id(request.character_id)?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (info, remain) = ctx
        .update_player()?
        .gift_favor(character_id, &request.itmes, tables)
        .map_err(invalid_favor)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFavorUseItem {
            info: Some(info),
            remain,
        },
    )
}

fn character_id(value: i64) -> NetworkResult<i32> {
    i32::try_from(value).map_err(|_| invalid_favor(FavorError::InvalidCharacterId(value)))
}

fn invalid_favor(error: FavorError) -> NetworkError {
    NetworkError::InvalidFavor(error.to_string())
}
