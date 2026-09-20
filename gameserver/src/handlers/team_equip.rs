use protocol::{
    cs::{
        DcNetWorkingParamTeamEquipEquip, DcNetWorkingParamTeamEquipList,
        DcNetWorkingParamTeamEquipLock, DcNetWorkingParamTeamEquipLvUp,
        DcNetWorkingParamTeamEquipSetCore, DcNetWorkingParamTeamEquipUnEquip,
        DcNetWorkingParamTeamEquipUnSetCore, DcNetWorkingResTeamEquipEquip,
        DcNetWorkingResTeamEquipList, DcNetWorkingResTeamEquipLock, DcNetWorkingResTeamEquipLvUp,
        DcNetWorkingResTeamEquipSetCore, DcNetWorkingResTeamEquipUnEquip,
        DcNetWorkingResTeamEquipUnSetCore,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTeamEquipList::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTeamEquipList {
            list: ctx.player()?.team_equip_details(),
        },
    )
}

pub async fn on_equip(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipEquip::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (equip_info, drop_fid) = ctx
        .update_player()?
        .equip_team_equip(request.te_id, request.fid, tables)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTeamEquipEquip {
            equip_info: Some(equip_info),
            drop_fid,
        },
    )
}

pub async fn on_unequip(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipUnEquip::decode(packet.payload.as_slice())?;
    let drop_fid = ctx
        .update_player()?
        .unequip_team_equip(request.te_id)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(&packet, DcNetWorkingResTeamEquipUnEquip { drop_fid })
}

pub async fn on_set_core(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipSetCore::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (equip_core, drop_core) = ctx
        .update_player()?
        .set_team_equip_core(request.te_id, request.core_id, request.pos, tables)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTeamEquipSetCore {
            equip_core: Some(equip_core),
            drop_core,
        },
    )
}

pub async fn on_unset_core(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipUnSetCore::decode(packet.payload.as_slice())?;
    let drop_core = ctx
        .update_player()?
        .unset_team_equip_core(request.te_id, request.pos)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(&packet, DcNetWorkingResTeamEquipUnSetCore { drop_core })
}

pub async fn on_lock(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipLock::decode(packet.payload.as_slice())?;
    let info = ctx
        .update_player()?
        .lock_team_equip(request.te_id, request.state)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(&packet, DcNetWorkingResTeamEquipLock { info: Some(info) })
}

pub async fn on_level_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTeamEquipLvUp::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (info, remain) = ctx
        .update_player()?
        .level_up_team_equip(request.te_id, &request.items, &request.cost_teid, tables)
        .map_err(invalid_team_equip)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTeamEquipLvUp {
            info: Some(info),
            remain,
            cost_teid: request.cost_teid,
        },
    )
}

fn invalid_team_equip(error: crate::logic::TeamEquipError) -> NetworkError {
    NetworkError::InvalidTeamEquip(error.to_string())
}
