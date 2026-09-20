use protocol::{
    cs::{
        DcNetWorkingParamEquipsEquipDelGroup, DcNetWorkingParamEquipsEquipGetGroup,
        DcNetWorkingParamEquipsEquipSetGroup, DcNetWorkingParamEquipsLevelUp,
        DcNetWorkingParamEquipsList, DcNetWorkingParamEquipsLock,
        DcNetWorkingParamEquipsRandomWordRefresh, DcNetWorkingParamEquipsRandomWordReplace,
        DcNetWorkingParamEquipsSet, DcNetWorkingParamEquipsSetEquips, DcNetWorkingParamEquipsSwap,
        DcNetWorkingParamEquipsUnset, DcNetWorkingResEquipsEquipDelGroup,
        DcNetWorkingResEquipsEquipGetGroup, DcNetWorkingResEquipsEquipSetGroup,
        DcNetWorkingResEquipsLevelUp, DcNetWorkingResEquipsList, DcNetWorkingResEquipsLock,
        DcNetWorkingResEquipsRandomWordRefresh, DcNetWorkingResEquipsRandomWordReplace,
        DcNetWorkingResEquipsSet, DcNetWorkingResEquipsSetEquips, DcNetWorkingResEquipsSwap,
        DcNetWorkingResEquipsUnset,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamEquipsList::decode(request.payload.as_slice())?;
    let list = ctx.player()?.equips.clone();
    ctx.send_reply(&request, DcNetWorkingResEquipsList { list })
}

pub async fn on_group_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamEquipsEquipGetGroup::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsEquipGetGroup {
            groups: ctx.player()?.equipment_groups.clone(),
        },
    )
}

pub async fn on_set_group(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsEquipSetGroup::decode(packet.payload.as_slice())?;
    let group = request
        .equips_group
        .ok_or_else(|| NetworkError::InvalidEquipment("missing equipment group".into()))?;
    let (equips_group, equips) = ctx
        .update_player()?
        .save_equipment_group(group)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsEquipSetGroup {
            equips_group: Some(equips_group),
            equips,
        },
    )
}

pub async fn on_delete_group(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsEquipDelGroup::decode(packet.payload.as_slice())?;
    let group = request
        .group
        .ok_or_else(|| NetworkError::InvalidEquipment("missing equipment group id".into()))?;
    ctx.update_player()?
        .delete_equipment_group(group.id)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsEquipDelGroup { group: Some(group) },
    )
}

pub async fn on_set_equips(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsSetEquips::decode(packet.payload.as_slice())?;
    let eq_set_ress = ctx
        .update_player()?
        .set_equipment_group(request.game_role_id, &request.user_equip_ids)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsSetEquips {
            game_role_id: request.game_role_id,
            eq_set_ress,
        },
    )
}

pub async fn on_level_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsLevelUp::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (equip, items, returns) = ctx
        .update_player()?
        .level_up_equipment(request.user_equip_id, &request.itmes, tables)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsLevelUp {
            equip: Some(equip),
            items,
            returns,
        },
    )
}

pub async fn on_random_word_refresh(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsRandomWordRefresh::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (word_id, remains) = ctx
        .update_player()?
        .refresh_equipment_word(request.user_equip_id, request.idx, tables)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsRandomWordRefresh { word_id, remains },
    )
}

pub async fn on_random_word_replace(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsRandomWordReplace::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let equip = ctx
        .update_player()?
        .replace_equipment_word(
            request.user_equip_id,
            request.idx,
            request.word_id,
            request.state,
            tables,
        )
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsRandomWordReplace { equip: Some(equip) },
    )
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsSet::decode(packet.payload.as_slice())?;
    let result = ctx
        .update_player()?
        .set_equipment(request.game_role_id, request.user_equip_id, request.pos)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsSet {
            equip_set_res: Some(result),
        },
    )
}

pub async fn on_unset(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsUnset::decode(packet.payload.as_slice())?;
    let equips = ctx
        .update_player()?
        .unset_equipment(request.game_role_id, request.pos)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResEquipsUnset { equips })
}

pub async fn on_swap(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsSwap::decode(packet.payload.as_slice())?;
    let (equip1, equip2) = ctx
        .update_player()?
        .swap_equipment(request.id1, request.id2)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEquipsSwap {
            equip1: Some(equip1),
            equip2: Some(equip2),
        },
    )
}

pub async fn on_lock(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEquipsLock::decode(packet.payload.as_slice())?;
    let info = request
        .info
        .ok_or_else(|| NetworkError::InvalidEquipment("missing lock info".into()))?;
    ctx.update_player()?
        .lock_equipment(info.user_equip_id, info.do_lock)
        .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResEquipsLock { info: Some(info) })
}

fn invalid(error: crate::logic::EquipmentError) -> NetworkError {
    NetworkError::InvalidEquipment(error.to_string())
}
