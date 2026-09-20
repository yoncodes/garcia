use protocol::{
    cs::{
        DcNetWorkingParamRBoxInfoList, DcNetWorkingParamRBoxTakeReward,
        DcNetWorkingParamTBoxInfoList, DcNetWorkingParamTBoxTakeReward,
        DcNetWorkingResRBoxInfoList, DcNetWorkingResTBoxInfoList, DcNetWorkingResTBoxTakeReward,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTBoxInfoList::decode(packet.payload.as_slice())?;
    let box_infos = ctx.player()?.reward_boxes();
    ctx.send_reply(&packet, DcNetWorkingResTBoxInfoList { box_infos })
}

pub async fn on_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTBoxTakeReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let reward = ctx
        .update_player()?
        .claim_reward_box(
            &request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidRewardBox(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTBoxTakeReward {
            reward: Some(reward),
        },
    )
}

pub async fn on_refresh_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRBoxInfoList::decode(packet.payload.as_slice())?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRBoxInfoList {
            box_infos: Vec::new(),
        },
    )
}

pub async fn on_refresh_box_reward(
    _ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRBoxTakeReward::decode(packet.payload.as_slice())?;
    Err(NetworkError::InvalidRewardBox(format!(
        "refresh-box instance {} was not issued by this server",
        request.id
    )))
}
