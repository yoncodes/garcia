use protocol::{
    cs::{
        DcNetWorkingParamMissonsList, DcNetWorkingParamMissonsTakeReward,
        DcNetWorkingResMissonsList, DcNetWorkingResMissonsTakeReward,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMissonsList::decode(request.payload.as_slice())?;
    let missions = ctx.player()?.missions.clone();
    ctx.send_reply(&request, DcNetWorkingResMissonsList { missions })
}

pub async fn on_take_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamMissonsTakeReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (mission_info, reward_info) = ctx
        .update_player()?
        .claim_mission(
            request.misson_id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidMission(error.to_string()))?;
    let red_dots = ctx.player()?.reward_item_red_dots(&reward_info, tables);
    super::red_dot::push_each(ctx, red_dots)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResMissonsTakeReward {
            reward_info: Some(reward_info),
            mission_info: Some(mission_info),
        },
    )
}
