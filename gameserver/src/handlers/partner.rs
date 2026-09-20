use protocol::{
    cs::{
        DcNetWorkingParamPartnerBrek, DcNetWorkingParamPartnerChange,
        DcNetWorkingParamPartnerImpress, DcNetWorkingParamPartnerList,
        DcNetWorkingParamPartnerLock, DcNetWorkingParamPartnerLvUp,
        DcNetWorkingParamPartnerSkillUp, DcNetWorkingParamPartnerSwap,
        DcNetWorkingParamPartnerUnset, DcNetWorkingResPartnerBrek, DcNetWorkingResPartnerChange,
        DcNetWorkingResPartnerImpress, DcNetWorkingResPartnerList, DcNetWorkingResPartnerLock,
        DcNetWorkingResPartnerLvUp, DcNetWorkingResPartnerSkillUp, DcNetWorkingResPartnerSwap,
        DcNetWorkingResPartnerUnset,
    },
    prost::Message,
};

use crate::{
    logic::{
        PartnerLevelOutcome, PartnerResonanceOutcome, PartnerSwapOutcome, PartnerUnsetOutcome,
    },
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamPartnerList::decode(request.payload.as_slice())?;
    let list = ctx.player()?.partners.clone();
    ctx.send_reply(&request, DcNetWorkingResPartnerList { list })
}

pub async fn on_change(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerChange::decode(packet.payload.as_slice())?;
    let outcome = ctx
        .update_player()?
        .change_partner(request.id, request.game_role_id)
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerChange {
            partner: Some(outcome.partner),
            roleinfo: Some(outcome.role_info),
            unset_roleinfo: outcome.unset_role_info,
        },
    )
}

pub async fn on_unset(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerUnset::decode(packet.payload.as_slice())?;
    let PartnerUnsetOutcome { partner, role_info } = ctx
        .update_player()?
        .unset_partner(request.id)
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerUnset {
            partner: Some(partner),
            roleinfo: Some(role_info),
        },
    )
}

pub async fn on_swap(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerSwap::decode(packet.payload.as_slice())?;
    let PartnerSwapOutcome {
        role_info1,
        role_info2,
    } = ctx
        .update_player()?
        .swap_partners(request.id1, request.id2)
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerSwap {
            roleinfo1: Some(role_info1),
            roleinfo2: Some(role_info2),
        },
    )
}

pub async fn on_lock(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerLock::decode(packet.payload.as_slice())?;
    let partner = ctx
        .update_player()?
        .lock_partner(request.id, request.is_lock)
        .map_err(invalid_role)?;
    ctx.send_reply(&packet, DcNetWorkingResPartnerLock { res: Some(partner) })
}

pub async fn on_level_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerLvUp::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let PartnerLevelOutcome {
        partner,
        cost_remain,
        remain,
    } = ctx
        .update_player()?
        .level_up_partner(request.id, &request.items, tables)
        .map_err(invalid_progression)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerLvUp {
            weap_info: Some(partner),
            cost_remain,
            remain,
        },
    )
}

pub async fn on_break(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerBrek::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (partner, remains) = ctx
        .update_player()?
        .break_partner(request.id, tables)
        .map_err(invalid_progression)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerBrek {
            partner: Some(partner),
            remains,
        },
    )
}

pub async fn on_skill_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerSkillUp::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (partner, cost_remain) = ctx
        .update_player()?
        .upgrade_partner_skill(request.id, tables)
        .map_err(invalid_progression)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerSkillUp {
            partner: Some(partner),
            cost_remain,
        },
    )
}

pub async fn on_impress(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPartnerImpress::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let PartnerResonanceOutcome {
        partner,
        consumed_partners,
        remains,
    } = ctx
        .update_player()?
        .resonate_partner(request.id, &request.cost_partner, tables)
        .map_err(invalid_progression)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPartnerImpress {
            partner: Some(partner),
            cost_partner: consumed_partners,
            remain: remains,
        },
    )
}

fn invalid_role(error: crate::logic::RoleMutationError) -> NetworkError {
    NetworkError::InvalidRoleMutation(error.to_string())
}

fn invalid_progression(error: crate::logic::PartnerProgressError) -> NetworkError {
    NetworkError::InvalidPartnerProgression(error.to_string())
}
