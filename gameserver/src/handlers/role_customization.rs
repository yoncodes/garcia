use protocol::{
    cs::{
        DcNetWorkingParamRolesChangeAppear, DcNetWorkingParamRolesChangeElement,
        DcNetWorkingParamRolesChangeElement4Call, DcNetWorkingParamRolesMcSwitch,
        DcNetWorkingParamRolesProfile, DcNetWorkingParamUserChangeBannerGril,
        DcNetWorkingResRolesChangeAppear, DcNetWorkingResRolesChangeElement,
        DcNetWorkingResRolesChangeElement4Call, DcNetWorkingResRolesMcSwitch,
        DcNetWorkingResRolesProfile, DcNetWorkingResUserChangeBannerGril,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_change_appearance(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesChangeAppear::decode(packet.payload.as_slice())?;
    let change = request
        .res
        .ok_or_else(|| NetworkError::InvalidRoleMutation("missing appearance change".into()))?;
    let res = ctx
        .update_player()?
        .change_role_appearance(change)
        .map_err(invalid)?;
    ctx.send_reply(&packet, DcNetWorkingResRolesChangeAppear { res: Some(res) })
}

pub async fn on_change_element(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesChangeElement::decode(packet.payload.as_slice())?;
    let info = ctx
        .update_player()?
        .change_role_element(request.game_role_id, request.element, false)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesChangeElement { info: Some(info) },
    )
}

pub async fn on_change_call_element(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesChangeElement4Call::decode(packet.payload.as_slice())?;
    let info = ctx
        .update_player()?
        .change_role_element(request.game_role_id, request.element, true)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesChangeElement4Call { info: Some(info) },
    )
}

pub async fn on_profiles(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRolesProfile::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesProfile {
            profile_list: ctx.player()?.role_profiles(),
        },
    )
}

pub async fn on_change_banner(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamUserChangeBannerGril::decode(packet.payload.as_slice())?;
    let banner_girl = ctx
        .update_player()?
        .change_banner_girl(request.banner_girl)
        .map_err(invalid)?;
    ctx.send_reply(&packet, DcNetWorkingResUserChangeBannerGril { banner_girl })
}

pub async fn on_mc_switch(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesMcSwitch::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let outcome = ctx
        .update_player()?
        .switch_main_character(request.game_role_id, tables)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesMcSwitch {
            pre_role: Some(outcome.previous),
            role: Some(outcome.current),
            drop_equips: outcome.dropped_equips,
            drop_partner: outcome.dropped_partner,
            drop_formation: outcome.changed_formations,
            drop_skillstones: outcome.dropped_skillstones,
        },
    )
}

fn invalid(error: crate::logic::RoleMutationError) -> NetworkError {
    NetworkError::InvalidRoleMutation(error.to_string())
}
