use protocol::{
    cs::{
        DcNetWorkingParamVersionChallengeAward, DcNetWorkingParamVersionChallengeList,
        DcNetWorkingParamVersionChallengeSett, DcNetWorkingParamVersionChallengeStageList,
        DcNetWorkingParamVersionChallengeStart, DcNetWorkingResVersionChallengeAward,
        DcNetWorkingResVersionChallengeList, DcNetWorkingResVersionChallengeSett,
        DcNetWorkingResVersionChallengeStageList, DcNetWorkingResVersionChallengeStart,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamVersionChallengeStart::decode(packet.payload.as_slice())?;
    ctx.player()?
        .start_version_challenge(
            request.id,
            &ctx.state.tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(&packet, DcNetWorkingResVersionChallengeStart { state: 0 })
}

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamVersionChallengeList::decode(packet.payload.as_slice())?;
    let list = ctx.player()?.version_challenges(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResVersionChallengeList { list })
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamVersionChallengeSett::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let info = ctx
        .update_player()?
        .settle_version_challenge(
            request.id,
            &request.pass_cond,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResVersionChallengeSett { info: Some(info) },
    )
}

pub async fn on_award(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamVersionChallengeAward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (reward, rewards) = ctx
        .update_player()?
        .claim_version_challenge_rewards(
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResVersionChallengeAward {
            reward: Some(reward),
            rewards,
        },
    )
}

pub async fn on_stage_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamVersionChallengeStageList::decode(packet.payload.as_slice())?;
    let list = ctx.player()?.version_challenge_stages(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResVersionChallengeStageList { list })
}

fn invalid(error: crate::logic::VersionChallengeError) -> NetworkError {
    NetworkError::InvalidVersionChallenge(error.to_string())
}
