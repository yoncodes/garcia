use protocol::cs::{
    DcNetWorkingNotifyBattlePassTask, DcNetWorkingParamBattlePassClaim,
    DcNetWorkingParamBattlePassInfo, DcNetWorkingParamBattlePassLevelUp,
    DcNetWorkingParamBattlePassTaskClaim, DcNetWorkingParamBattlePassTasks,
    DcNetWorkingResBattlePassClaim, DcNetWorkingResBattlePassInfo,
    DcNetWorkingResBattlePassLevelUp, DcNetWorkingResBattlePassTaskClaim,
    DcNetWorkingResBattlePassTasks,
};
use protocol::prost::Message;

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub(crate) fn push_task_updates(
    ctx: &mut HandlerContext,
    tasks: &[protocol::pbcommon::DcNetDataBattlePassTask],
) -> NetworkResult<()> {
    for task in tasks {
        let sends = if task.total > 0 && task.progress >= task.total {
            2
        } else {
            1
        };
        for _ in 0..sends {
            ctx.push(
                protocol::proids::NotifyId::DcNetWorkingNotifyBattlePassTask as u16,
                DcNetWorkingNotifyBattlePassTask { task: Some(*task) },
            )?;
        }
    }
    Ok(())
}

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamBattlePassInfo::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let Some((info, _)) = ctx.update_player()?.battle_pass_info(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    ) else {
        return ctx.send_reply(&packet, DcNetWorkingResBattlePassInfo::default());
    };
    ctx.send_reply(
        &packet,
        DcNetWorkingResBattlePassInfo {
            id: info.id,
            exp_week_limit: info.exp_week_limit,
            exp_per_level: info.exp_per_level,
            level_max: info.level_max,
            start_time: info.start_time,
            end_time: info.end_time,
            advanced_reward: info.advanced_reward.into_iter().collect(),
            premium_reward: info.premium_reward.into_iter().collect(),
            battle_pass: Some(info.battle_pass),
        },
    )
}

pub async fn on_tasks(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamBattlePassTasks::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let Some((tasks, weekly_remain_sec, daily_remain_sec, _)) =
        ctx.update_player()?.battle_pass_tasks(
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
    else {
        return ctx.send_reply(&packet, DcNetWorkingResBattlePassTasks::default());
    };
    ctx.send_reply(
        &packet,
        DcNetWorkingResBattlePassTasks {
            tasks,
            weekly_remain_sec,
            daily_remain_sec,
        },
    )
}

pub async fn on_task_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBattlePassTaskClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (task, battle_pass) = ctx
        .update_player()?
        .claim_battle_pass_tasks(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidBattlePass(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBattlePassTaskClaim {
            task,
            battle_pass: Some(battle_pass),
        },
    )
}

pub async fn on_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBattlePassClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let zone_offset = common::config().server.zone_offset;
    let (reward, battle_pass) = ctx
        .update_player()?
        .claim_battle_pass_rewards(request.level, tables, now, zone_offset)
        .map_err(|error| NetworkError::InvalidBattlePass(error.to_string()))?;
    let task_updates =
        ctx.update_player()?
            .battle_pass_reward_updates(&reward, tables, now, zone_offset);
    push_task_updates(ctx, &task_updates)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBattlePassClaim {
            reward: Some(reward),
            battle_pass: Some(battle_pass),
        },
    )
}

pub async fn on_level_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBattlePassLevelUp::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (remain, battle_pass) = ctx
        .update_player()?
        .level_up_battle_pass(
            request.add_level,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidBattlePass(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBattlePassLevelUp {
            remain: Some(remain),
            battle_pass: Some(battle_pass),
        },
    )
}
