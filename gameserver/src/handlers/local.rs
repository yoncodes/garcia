use protocol::{
    cs::{
        DcNetWorkingParamLocalList, DcNetWorkingParamLocalSet, DcNetWorkingResLocalList,
        DcNetWorkingResLocalSet,
    },
    pbcommon::DcNetDataLocal,
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamLocalList::decode(request.payload.as_slice())?;
    let player = ctx.player()?;
    let response = DcNetWorkingResLocalList {
        region: player.region,
        locals: player.locals.clone(),
    };
    ctx.send_reply(&request, response)
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamLocalSet::decode(packet.payload.as_slice())?;
    let player = ctx.update_player()?;
    player.set_local(DcNetDataLocal {
        region: request.region,
        local: request.local,
        local2: request.local2,
    });
    let response = DcNetWorkingResLocalSet {
        region: player.region,
        locals: player.locals.clone(),
    };
    ctx.send_reply(&packet, response)
}
