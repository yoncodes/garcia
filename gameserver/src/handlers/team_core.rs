use protocol::{
    cs::{
        DcNetWorkingParamTeamCoreCompose, DcNetWorkingParamTeamCoreList,
        DcNetWorkingParamTeamCoreLock, DcNetWorkingResTeamCoreCompose, DcNetWorkingResTeamCoreList,
        DcNetWorkingResTeamCoreLock,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTeamCoreList::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTeamCoreList {
            list: ctx.player()?.team_cores.clone(),
        },
    )
}

pub async fn on_compose(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamCoreCompose::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (list, remains) = ctx
        .update_player()?
        .compose_team_cores(request.id, request.num, tables)
        .map_err(invalid_team_core)?;
    ctx.send_reply(&packet, DcNetWorkingResTeamCoreCompose { list, remains })
}

pub async fn on_lock(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamCoreLock::decode(packet.payload.as_slice())?;
    let info = ctx
        .update_player()?
        .lock_team_core(request.cid, request.state)
        .map_err(invalid_team_core)?;
    ctx.send_reply(&packet, DcNetWorkingResTeamCoreLock { info: Some(info) })
}

fn invalid_team_core(error: crate::logic::TeamCoreError) -> NetworkError {
    NetworkError::InvalidTeamCore(error.to_string())
}
