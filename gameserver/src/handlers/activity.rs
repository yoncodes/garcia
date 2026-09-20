use protocol::cs::{
    DcNetWorkingNotifyAct7Day, DcNetWorkingParamActivity7DayClaim,
    DcNetWorkingParamActivity7DayList, DcNetWorkingParamActivity7DayPointClaim,
    DcNetWorkingParamActivityLevelUpList, DcNetWorkingParamActivityLevelUpReward,
    DcNetWorkingParamActivityLimitedLevelUpList, DcNetWorkingParamActivityLimitedLevelUpReward,
    DcNetWorkingParamActivityOpenList, DcNetWorkingParamCheckinCheck, DcNetWorkingParamCheckinInfo,
    DcNetWorkingParamCheckinReward, DcNetWorkingParamVersionActivityInfo,
    DcNetWorkingParamVersionTaskClaim, DcNetWorkingParamVersionTaskList,
    DcNetWorkingResActivity7DayClaim, DcNetWorkingResActivity7DayList,
    DcNetWorkingResActivity7DayPointClaim, DcNetWorkingResActivityLevelUpList,
    DcNetWorkingResActivityLevelUpReward, DcNetWorkingResActivityLimitedLevelUpList,
    DcNetWorkingResActivityLimitedLevelUpReward, DcNetWorkingResActivityOpenList,
    DcNetWorkingResCheckinCheck, DcNetWorkingResCheckinInfo, DcNetWorkingResCheckinReward,
    DcNetWorkingResVersionActivityInfo, DcNetWorkingResVersionTaskClaim,
    DcNetWorkingResVersionTaskList,
};
use protocol::pbcommon::DcNetDataAct7Day;
use protocol::prost::Message;

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub(crate) fn push_seven_day_updates(
    ctx: &mut HandlerContext,
    updates: &[DcNetDataAct7Day],
) -> NetworkResult<()> {
    for info in updates {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyAct7Day as u16,
            DcNetWorkingNotifyAct7Day { info: Some(*info) },
        )?;
    }
    Ok(())
}

pub async fn on_open_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamActivityOpenList::decode(packet.payload.as_slice())?;
    let ainfos = ctx.player()?.open_activities(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResActivityOpenList { ainfos })
}
pub async fn on_limited_level_up_list(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityLimitedLevelUpList::decode(packet.payload.as_slice())?;
    let player = ctx.player()?;
    let is_can = player.is_limited_level_activity_available(
        request.aid,
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    let ids = player.activity_limited_level_rewards.clone();
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityLimitedLevelUpList { is_can, ids },
    )
}

pub async fn on_limited_level_up_reward(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityLimitedLevelUpReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let take_res = ctx
        .update_player()?
        .claim_limited_level_reward(
            request.sid,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidActivity(error.to_string()))?;
    let red_dots = ctx.player()?.reward_item_red_dots(&take_res, tables);
    super::red_dot::push_each(ctx, red_dots)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityLimitedLevelUpReward {
            take_res: Some(take_res),
        },
    )
}
pub async fn on_check_in_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamCheckinInfo::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (statuses, _) = ctx.update_player()?.check_in_status(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResCheckinInfo { cinfos: statuses })
}

pub async fn on_check_in_check(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamCheckinCheck::decode(packet.payload.as_slice())?;
    ctx.send_reply(&packet, DcNetWorkingResCheckinCheck { code: 0 })
}

pub async fn on_check_in_reward(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamCheckinReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let take_res = ctx
        .update_player()?
        .claim_check_in_reward(
            request.aid,
            request.get_day,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidCheckin(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResCheckinReward {
            take_res: Some(take_res),
        },
    )
}
pub async fn on_level_up_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamActivityLevelUpList::decode(packet.payload.as_slice())?;
    let ids = ctx.player()?.activity_level_rewards.clone();
    ctx.send_reply(&packet, DcNetWorkingResActivityLevelUpList { ids })
}

pub async fn on_level_up_reward(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivityLevelUpReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let take_res = ctx
        .update_player()?
        .claim_activity_level_reward(
            request.sid,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidActivity(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivityLevelUpReward {
            take_res: Some(take_res),
        },
    )
}
pub async fn on_seven_day_list(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamActivity7DayList::decode(packet.payload.as_slice())?;
    let (list, point_claimed) = ctx.player()?.seven_day_activity_status(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
    );
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivity7DayList {
            list,
            point_claimed,
        },
    )
}

pub async fn on_seven_day_claim(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivity7DayClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (act_info, res) = ctx
        .update_player()?
        .claim_seven_day_activity_reward(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidActivity(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivity7DayClaim {
            act_info: Some(act_info),
            res: Some(res),
        },
    )
}

pub async fn on_seven_day_point_claim(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamActivity7DayPointClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let res = ctx
        .update_player()?
        .claim_seven_day_activity_milestone(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidActivity(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResActivity7DayPointClaim {
            id: request.id,
            res: Some(res),
        },
    )
}
pub async fn on_version_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamVersionActivityInfo::decode(packet.payload.as_slice())?;
    let (aid, countdown) = ctx
        .player()?
        .active_version_activity(
            &ctx.state.tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .unwrap_or_default();
    ctx.send_reply(
        &packet,
        DcNetWorkingResVersionActivityInfo { aid, countdown },
    )
}

pub async fn on_version_task_list(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamVersionTaskList::decode(packet.payload.as_slice())?;
    let tasks = ctx.player()?.version_tasks(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResVersionTaskList { tasks })
}

pub async fn on_version_task_claim(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamVersionTaskClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (task, reward) = ctx
        .update_player()?
        .claim_version_task(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidActivity(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResVersionTaskClaim {
            task: Some(task),
            reward: Some(reward),
        },
    )
}
