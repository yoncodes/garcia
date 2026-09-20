use protocol::{
    cs::{
        DcNetWorkingParamFormationAdd, DcNetWorkingParamFormationCurSet,
        DcNetWorkingParamFormationDel, DcNetWorkingParamFormationList,
        DcNetWorkingParamFormationSet, DcNetWorkingResFormationAdd, DcNetWorkingResFormationCurSet,
        DcNetWorkingResFormationDel, DcNetWorkingResFormationList, DcNetWorkingResFormationSet,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamFormationList::decode(request.payload.as_slice())?;
    let list = ctx.player()?.formations.clone();
    ctx.send_reply(&request, DcNetWorkingResFormationList { list })
}

pub async fn on_add(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamFormationAdd::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let formation = ctx
        .update_player()?
        .add_formation(tables)
        .map_err(|error| NetworkError::InvalidFormation(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFormationAdd {
            formation: Some(formation),
        },
    )
}

pub async fn on_delete(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFormationDel::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?
        .delete_formation(request.fid, tables)
        .map_err(|error| NetworkError::InvalidFormation(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResFormationDel { fid: request.fid })
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFormationSet::decode(packet.payload.as_slice())?;
    let formation = request
        .formation
        .ok_or_else(|| NetworkError::InvalidFormation("missing formation".into()))?;
    let state = ctx.state.clone();
    ctx.update_player()?
        .set_formation(formation.clone(), &state.tables)
        .map_err(|error| NetworkError::InvalidFormation(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFormationSet {
            formation: Some(formation),
        },
    )
}

pub async fn on_select(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFormationCurSet::decode(packet.payload.as_slice())?;
    ctx.update_player()?
        .select_formation(request.fid)
        .map_err(|error| NetworkError::InvalidFormation(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResFormationCurSet { fid: request.fid })
}
