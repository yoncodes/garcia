use protocol::{
    cs::{
        DcNetWorkingParamNaming, DcNetWorkingParamUserGetInfo, DcNetWorkingParamUserGetPhyInfo,
        DcNetWorkingResNaming, DcNetWorkingResUserGetInfo, DcNetWorkingResUserGetPhyInfo,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_stamina_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamUserGetPhyInfo::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let info = ctx
        .update_player()?
        .refresh_heat(tables, common::time::ServerTime::now_seconds_i32())
        .map_err(|error| NetworkError::InvalidHeatExchange(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResUserGetPhyInfo {
            item: Some(info.item),
            cd_time: info.cd_time,
        },
    )
}

pub async fn on_rename(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamNaming::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let remains = ctx
        .update_player()?
        .rename(request.name_type, request.name_str, request.gender, tables)
        .map_err(|error| NetworkError::InvalidNaming(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResNaming { code: 1, remains })
}

pub async fn on_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamUserGetInfo::decode(request.payload.as_slice())?;
    let player = ctx.player()?;
    let now = common::time::ServerTime::now_seconds_i32();
    let response = DcNetWorkingResUserGetInfo {
        level: player.level,
        gameplay_id: player.gameplay_id,
        // The live server leaves both account-facing strings unset. The client
        // receives its numeric identity during ConnectGameServer instead.
        uniq_show_id: String::new(),
        username: String::new(),
        nickname: player.nickname.clone(),
        last_login_time: player.last_login_time,
        created_at: player.created_at,
        cur_form: player.cur_form,
        profile_avatar: player.profile_avatar,
        profile_card: player.profile_card,
        profile_title: player.profile_title,
        profile_frame: player.profile_frame,
        banner_gril: player.banner_girl,
        world_lv: player.world_level(&ctx.state.tables),
        favor_num: player.favor_touches(
            &ctx.state.tables,
            now,
            common::config().server.zone_offset,
        ),
        gender: player.gender(&ctx.state.tables),
        daily_reward_taken: player.is_region_ticket_taken(
            &ctx.state.tables,
            now,
            common::config().server.zone_offset,
        ),
        ..Default::default()
    };
    ctx.send_reply(&request, response)
}
