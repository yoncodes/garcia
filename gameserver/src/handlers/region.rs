use protocol::{
    cs::{
        DcNetWorkingParamRegionInfo, DcNetWorkingParamRegionList,
        DcNetWorkingParamRegionTakeDailyReward, DcNetWorkingParamRegionTakeTask,
        DcNetWorkingResRegionInfo, DcNetWorkingResRegionList, DcNetWorkingResRegionTakeDailyReward,
        DcNetWorkingResRegionTakeTask,
    },
    prost::Message,
};

pub async fn on_take_daily_reward(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamRegionTakeDailyReward::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let state = ctx.state.clone();
    let tables = &state.tables;
    let item = ctx
        .update_player()?
        .claim_region_daily_ticket(tables, now, common::config().server.zone_offset)
        .map_err(|error| NetworkError::InvalidRegion(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRegionTakeDailyReward { reward: vec![item] },
    )
}

pub async fn on_take_task(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    let param = DcNetWorkingParamRegionTakeTask::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (task, remains) = ctx
        .update_player()?
        .accept_region_task(
            param.id,
            param.task_group_id,
            tables,
            now,
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidRegion(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRegionTakeTask {
            id: param.id,
            task: Some(task),
            remains,
        },
    )
}

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    let param = DcNetWorkingParamRegionInfo::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    ctx.update_player()?.refresh_daily_world(now);
    let info = ctx
        .player()?
        .region_info(
            param.id,
            &ctx.state.tables,
            now,
            common::config().server.zone_offset,
        )
        .ok_or(NetworkError::UnknownRegion(param.id))?;
    ctx.send_reply(&request, DcNetWorkingResRegionInfo { info: Some(info) })
}

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRegionList::decode(request.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    ctx.update_player()?.refresh_daily_world(now);
    let player = ctx.player()?;
    let list = ctx
        .state
        .tables
        .city_regions
        .rows
        .iter()
        .filter_map(|region| {
            player.region_info(
                region.id,
                &ctx.state.tables,
                now,
                common::config().server.zone_offset,
            )
        })
        .collect();
    ctx.send_reply(&request, DcNetWorkingResRegionList { list })
}
