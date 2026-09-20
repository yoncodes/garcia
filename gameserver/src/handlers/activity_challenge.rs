use protocol::{
    cs::{
        DcNetWorkingParamActivityChallengesClaim, DcNetWorkingParamActivityChallengesList,
        DcNetWorkingParamActivityChallengesPointClaim, DcNetWorkingResActivityChallengesClaim,
        DcNetWorkingResActivityChallengesList, DcNetWorkingResActivityChallengesPointClaim,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityChallengesList::decode(packet.payload.as_slice())?;
    let (list, claimed_point_ids) = ctx
        .player()?
        .activity_challenges(request.activity_id, &ctx.state.tables)
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityChallengesList {
            list,
            claimed_point_ids,
        },
    )
}

pub async fn on_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityChallengesClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let reward = ctx
        .update_player()?
        .claim_activity_challenge(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityChallengesClaim {
            reward: Some(reward),
        },
    )
}

pub async fn on_point_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityChallengesPointClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let reward = ctx
        .update_player()?
        .claim_activity_challenge_point(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityChallengesPointClaim {
            reward: Some(reward),
        },
    )
}

fn invalid(error: crate::logic::ActivityChallengeError) -> NetworkError {
    NetworkError::InvalidActivityChallenge(error.to_string())
}
