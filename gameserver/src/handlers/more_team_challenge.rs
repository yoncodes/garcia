use protocol::{
    cs::{
        DcNetWorkingParamMoreTeamChallengeAward, DcNetWorkingParamMoreTeamChallengeGroupList,
        DcNetWorkingParamMoreTeamChallengeList, DcNetWorkingParamMoreTeamChallengeSett,
        DcNetWorkingParamMoreTeamChallengeStart, DcNetWorkingResMoreTeamChallengeAward,
        DcNetWorkingResMoreTeamChallengeGroupList, DcNetWorkingResMoreTeamChallengeList,
        DcNetWorkingResMoreTeamChallengeSett, DcNetWorkingResMoreTeamChallengeStart,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_group_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMoreTeamChallengeGroupList::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (ginfos, _) = ctx.update_player()?.more_team_challenge_groups(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(
        &packet,
        DcNetWorkingResMoreTeamChallengeGroupList { ginfos },
    )
}

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMoreTeamChallengeList::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (list, _) = ctx.update_player()?.more_team_challenges(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResMoreTeamChallengeList { list })
}

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamMoreTeamChallengeStart::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?
        .start_more_team_challenge(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(&packet, DcNetWorkingResMoreTeamChallengeStart { state: 0 })
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamMoreTeamChallengeSett::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let info = ctx
        .update_player()?
        .settle_more_team_challenge(
            request.id,
            request.cost_time,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResMoreTeamChallengeSett { info: Some(info) },
    )
}

pub async fn on_award(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMoreTeamChallengeAward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (reward, rewards, _) = ctx.update_player()?.claim_more_team_challenge_rewards(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(
        &packet,
        DcNetWorkingResMoreTeamChallengeAward {
            reward: Some(reward),
            rewards,
        },
    )
}

fn invalid(error: crate::logic::MoreTeamChallengeError) -> NetworkError {
    NetworkError::InvalidMoreTeamChallenge(error.to_string())
}
