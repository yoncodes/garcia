use protocol::{
    cs::{
        DcNetWorkingParamActivityBossChallengeDailyReward,
        DcNetWorkingParamActivityBossChallengeInfo, DcNetWorkingParamActivityBossChallengeRank,
        DcNetWorkingParamActivityBossChallengeReward, DcNetWorkingParamActivityBossChallengeSett,
        DcNetWorkingParamActivityBossChallengeStart,
        DcNetWorkingResActivityBossChallengeDailyReward, DcNetWorkingResActivityBossChallengeInfo,
        DcNetWorkingResActivityBossChallengeRank, DcNetWorkingResActivityBossChallengeReward,
        DcNetWorkingResActivityBossChallengeSett, DcNetWorkingResActivityBossChallengeStart,
    },
    pbcommon::DcNetDataActivityBossInfo,
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityBossChallengeInfo::decode(packet.payload.as_slice())?;
    let (boss_info, can_fight, day_num, _) = activity_info(ctx, request.aid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeInfo {
            boss_info: Some(boss_info),
            can_fight,
            day_num,
        },
    )
}

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityBossChallengeStart::decode(packet.payload.as_slice())?;
    let (_, can_fight, _, _) = activity_info(ctx, request.aid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeStart { can_fight },
    )
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityBossChallengeSett::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (curr_gameplay_id, score) = ctx
        .update_player()?
        .settle_activity_boss(
            request.aid,
            request.damage,
            request.role_ids,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            zone_offset(),
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeSett {
            curr_gameplay_id,
            score,
        },
    )
}

pub async fn on_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityBossChallengeReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let take_res = ctx
        .update_player()?
        .claim_activity_boss_reward(
            request.aid,
            request.sid,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeReward {
            take_res: Some(take_res),
        },
    )
}

pub async fn on_daily_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request =
        DcNetWorkingParamActivityBossChallengeDailyReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let take_res = ctx
        .update_player()?
        .claim_activity_boss_daily_rewards(
            request.aid,
            &request.ids,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeDailyReward {
            take_res: Some(take_res),
        },
    )
}

pub async fn on_rank(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityBossChallengeRank::decode(packet.payload.as_slice())?;
    activity_info(ctx, request.aid)?;
    let (ranks, cur_rank, role_ids) = ctx.player()?.activity_boss_rank();
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityBossChallengeRank {
            ranks,
            cur_rank,
            role_ids,
        },
    )
}

fn activity_info(
    ctx: &mut HandlerContext,
    aid: i32,
) -> NetworkResult<(DcNetDataActivityBossInfo, bool, i32, bool)> {
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?
        .activity_boss_info(
            aid,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            zone_offset(),
        )
        .map_err(invalid)
}

fn zone_offset() -> i32 {
    common::config().server.zone_offset
}

fn invalid(error: crate::logic::ActivityBossError) -> NetworkError {
    NetworkError::InvalidActivityBoss(error.to_string())
}
