use protocol::{
    cs::{
        DcNetWorkingParamDenseFogList, DcNetWorkingParamDenseFogSet,
        DcNetWorkingParamTpMapInfoList, DcNetWorkingParamTpMapSet, DcNetWorkingResDenseFogList,
        DcNetWorkingResDenseFogSet, DcNetWorkingResTpMapInfoList, DcNetWorkingResTpMapSet,
    },
    pbcommon::DcNetDataTpMapInfo,
    prost::Message,
};

pub async fn on_teleport_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTpMapInfoList::decode(packet.payload.as_slice())?;
    let tpmap_infos = ctx
        .player()?
        .tp_map
        .iter()
        .map(|(key, value)| DcNetDataTpMapInfo {
            key: key.clone(),
            value: value.clone(),
        })
        .collect();
    ctx.send_reply(&packet, DcNetWorkingResTpMapInfoList { tpmap_infos })
}

pub async fn on_teleport_update(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamTpMapSet::decode(packet.payload.as_slice())?;
    ctx.update_player()?.set_tp_map(request.key, request.value);
    ctx.send_reply(&packet, DcNetWorkingResTpMapSet { code: 0 })
}

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_dense_fog_list(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamDenseFogList::decode(request.payload.as_slice())?;
    let id = ctx.player()?.dense_fogs.clone();
    ctx.send_reply(&request, DcNetWorkingResDenseFogList { id })
}

pub async fn on_dense_fog_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamDenseFogSet::decode(packet.payload.as_slice())?;
    ctx.update_player()?.add_dense_fogs(request.id);
    ctx.send_reply(&packet, DcNetWorkingResDenseFogSet { code: 0 })
}
