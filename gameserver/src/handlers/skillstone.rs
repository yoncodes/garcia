use protocol::{
    cs::{
        DcNetWorkingParamSkillStoneList, DcNetWorkingParamSkillstonesLock,
        DcNetWorkingParamSkillstonesSet, DcNetWorkingParamSkillstonesSwap,
        DcNetWorkingParamSkillstonesUnset, DcNetWorkingResSkillStoneList,
        DcNetWorkingResSkillstonesLock, DcNetWorkingResSkillstonesSet,
        DcNetWorkingResSkillstonesSwap, DcNetWorkingResSkillstonesUnset,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamSkillStoneList::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResSkillStoneList {
            list: ctx.player()?.skillstones.clone(),
        },
    )
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamSkillstonesSet::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (stone, dropped_list) = ctx
        .update_player()?
        .set_skillstone(
            request.game_role_id,
            request.user_stone_id,
            request.pos,
            tables,
        )
        .map_err(invalid_skillstone)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResSkillstonesSet {
            skillstones: vec![stone],
            dropped_list,
        },
    )
}

pub async fn on_unset(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamSkillstonesUnset::decode(packet.payload.as_slice())?;
    let skillstones = ctx
        .update_player()?
        .unset_skillstone(request.game_role_id, request.pos)
        .map_err(invalid_skillstone)?;
    ctx.send_reply(&packet, DcNetWorkingResSkillstonesUnset { skillstones })
}

pub async fn on_lock(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamSkillstonesLock::decode(packet.payload.as_slice())?;
    let skillstones = ctx
        .update_player()?
        .lock_skillstone(request.user_stone_id, request.do_lock)
        .map_err(invalid_skillstone)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResSkillstonesLock {
            skillstones: Some(skillstones),
        },
    )
}

pub async fn on_swap(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamSkillstonesSwap::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (skillstone1, skillstone2) = ctx
        .update_player()?
        .swap_skillstones(request.id1, request.id2, tables)
        .map_err(invalid_skillstone)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResSkillstonesSwap {
            skillstone1: Some(skillstone1),
            skillstone2: Some(skillstone2),
        },
    )
}

fn invalid_skillstone(error: crate::logic::SkillStoneError) -> NetworkError {
    NetworkError::InvalidSkillStone(error.to_string())
}
