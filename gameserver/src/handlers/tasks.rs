use protocol::{
    cs::{
        DcNetWorkingNotifyInteractObjs, DcNetWorkingNotifyRegion, DcNetWorkingNotifyTaskCycle,
        DcNetWorkingNotifyWorldLevel, DcNetWorkingParamTask2Done, DcNetWorkingParamTaskCycleList,
        DcNetWorkingParamTaskCycleTakeActivity, DcNetWorkingParamTaskCycleTakeReward,
        DcNetWorkingParamTaskGoalSet, DcNetWorkingParamTaskList, DcNetWorkingResTask2Done,
        DcNetWorkingResTaskCycleList, DcNetWorkingResTaskCycleTakeActivity,
        DcNetWorkingResTaskCycleTakeReward, DcNetWorkingResTaskGoalSet, DcNetWorkingResTaskList,
    },
    pbcommon::{DcNetDataMissonInfo, DcNetDataTaskChanged, DcNetDataTaskCycle, Reddot},
    prost::Message,
};

use crate::logic::{TaskCompleteOutcome, TaskProgressOutcome};
use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTaskList::decode(request.payload.as_slice())?;
    let tasklist = ctx.player()?.tasks.clone();
    ctx.send_reply(&request, DcNetWorkingResTaskList { tasklist })
}

pub async fn on_goal_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTaskGoalSet::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let task_id = ctx
        .update_player()?
        .set_traced_task_group(request.task_id, tables)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResTaskGoalSet { task_id })
}

pub async fn on_complete(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTask2Done::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let outcome = ctx
        .update_player()?
        .complete_task(request.task_id, request.select, tables, now)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
    let level_progress = ctx
        .update_player()?
        .advance_level_tasks(tables, now)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
    push_completion_body(ctx, &outcome, now)?;
    push_progress(ctx, level_progress, now)?;
    push_level_update(ctx, &outcome)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTask2Done {
            cur_task: Some(outcome.current),
            reward: Some(outcome.rewards),
        },
    )
}

pub(crate) fn push_completion(
    ctx: &mut HandlerContext,
    outcome: &TaskCompleteOutcome,
    now: i32,
) -> NetworkResult<()> {
    push_completion_body(ctx, outcome, now)?;
    push_level_update(ctx, outcome)
}

fn push_completion_body(
    ctx: &mut HandlerContext,
    outcome: &TaskCompleteOutcome,
    now: i32,
) -> NetworkResult<()> {
    let state = ctx.state.clone();
    let tables = &state.tables;
    let region_update = if let Some((region_id, exp_item)) = outcome.region_reward {
        Some(DcNetWorkingNotifyRegion {
            region: ctx.player()?.region_info(
                region_id,
                tables,
                now,
                common::config().server.zone_offset,
            ),
            exp_item: Some(exp_item),
        })
    } else {
        None
    };
    let mut changed = outcome.changed.clone();
    changed.push(outcome.current);
    push_features(ctx, &outcome.feat_updates)?;
    for object in &outcome.interact_updates {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyInteractObjs as u16,
            DcNetWorkingNotifyInteractObjs {
                obj: Some(object.clone()),
            },
        )?;
    }
    for message in &outcome.sms_updates {
        ctx.push(
            protocol::proids::NotifyId::DcNetDataSms as u16,
            message.clone(),
        )?;
    }
    ctx.push(
        protocol::proids::NotifyId::DcNetDataTaskChanged as u16,
        DcNetDataTaskChanged {
            tasklist: changed,
            rewards: Some(outcome.rewards.clone()),
        },
    )?;
    super::achievement::push_updates(ctx, &outcome.achievement_updates)?;
    super::archive::push_updates(ctx, &outcome.archive_updates)?;
    for album in &outcome.album_updates {
        ctx.push(protocol::proids::NotifyId::DcNetDataAlbums as u16, *album)?;
        super::red_dot::push_one(ctx, Reddot::Album, album.id)?;
    }
    push_missions(ctx, &outcome.mission_updates)?;
    if let Some(update) = region_update {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyRegion as u16,
            update,
        )?;
    }
    Ok(())
}

fn push_level_update(ctx: &mut HandlerContext, outcome: &TaskCompleteOutcome) -> NetworkResult<()> {
    if let Some(world_level) = outcome.world_level {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyWorldLevel as u16,
            DcNetWorkingNotifyWorldLevel {
                world_level,
                updata: Some(outcome.level_up_data.clone().unwrap_or_default()),
            },
        )
    } else if let Some(update) = outcome
        .level_up_data
        .as_ref()
        .filter(|update| update.is_lv_up)
    {
        ctx.push(
            protocol::proids::NotifyId::DcNetDataAttrUpData as u16,
            update.clone(),
        )
    } else {
        Ok(())
    }
}

pub(crate) fn push_features(
    ctx: &mut HandlerContext,
    features: &[protocol::pbcommon::DcNetDataFeat],
) -> NetworkResult<()> {
    for feature in features {
        ctx.push(protocol::proids::NotifyId::DcNetDataFeat as u16, *feature)?;
        super::red_dot::push_one(ctx, Reddot::Feat, feature.id)?;
    }
    Ok(())
}

pub(crate) fn push_task_changes(
    ctx: &mut HandlerContext,
    tasks: &[protocol::pbcommon::DcNetDataTaskStatus],
) -> NetworkResult<()> {
    if tasks.is_empty() {
        return Ok(());
    }
    ctx.push(
        protocol::proids::NotifyId::DcNetDataTaskChanged as u16,
        DcNetDataTaskChanged {
            tasklist: tasks.to_vec(),
            rewards: None,
        },
    )
}

pub(crate) fn push_missions(
    ctx: &mut HandlerContext,
    missions: &[DcNetDataMissonInfo],
) -> NetworkResult<()> {
    for mission in missions {
        ctx.push(
            protocol::proids::NotifyId::DcNetDataMissonInfo as u16,
            *mission,
        )?;
    }
    Ok(())
}

pub(crate) fn push_daily_updates(
    ctx: &mut HandlerContext,
    updates: &[DcNetDataTaskCycle],
) -> NetworkResult<()> {
    for task in updates {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyTaskCycle as u16,
            DcNetWorkingNotifyTaskCycle { task: Some(*task) },
        )?;
    }
    Ok(())
}

pub(crate) fn push_progress(
    ctx: &mut HandlerContext,
    outcomes: Vec<TaskProgressOutcome>,
    now: i32,
) -> NetworkResult<()> {
    for outcome in outcomes {
        // PROVEN by live story-port captures (for example sequences 714/715):
        // progress uses the bare status payload under cmd 10018, while completion
        // immediately follows with the DcNetDataTaskChanged wrapper.
        ctx.push(
            protocol::proids::NotifyId::DcNetDataTaskChanged as u16,
            outcome.progress,
        )?;
        if let Some(completion) = outcome.completion {
            push_completion(ctx, &completion, now)?;
        }
    }
    Ok(())
}

pub async fn on_cycle_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTaskCycleList::decode(packet.payload.as_slice())?;
    let player = ctx.player()?;
    let response = DcNetWorkingResTaskCycleList {
        activity: player.daily_activity,
        progress: player.daily_reward_progress,
        list: player.daily_tasks.clone(),
    };
    ctx.send_reply(&packet, response)
}

pub async fn on_cycle_take_activity(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamTaskCycleTakeActivity::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (id, activity, progress) = ctx
        .update_player()?
        .claim_daily_activity(request.id, tables)
        .map_err(|error| NetworkError::InvalidDailyTask(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTaskCycleTakeActivity {
            id,
            activity,
            progress,
        },
    )
}

pub async fn on_cycle_take_reward(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamTaskCycleTakeReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let outcome = ctx
        .update_player()?
        .claim_daily_rewards(tables, now)
        .map_err(|error| NetworkError::InvalidDailyTask(error.to_string()))?;
    let mission_updates = ctx.update_player()?.advance_level_missions(now);
    let level_progress = ctx
        .update_player()?
        .advance_level_tasks(tables, now)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
    let player = ctx.player()?;
    let response = DcNetWorkingResTaskCycleTakeReward {
        activity: player.daily_activity,
        progress: player.daily_reward_progress,
        reward: Some(outcome.rewards),
        level_up_data: Some(outcome.level_up_data.clone()),
    };
    push_missions(ctx, &mission_updates)?;
    ctx.push(
        protocol::proids::NotifyId::DcNetDataAttrUpData as u16,
        outcome.level_up_data,
    )?;
    super::archive::push_updates(ctx, &outcome.archive_updates)?;
    push_progress(ctx, level_progress, now)?;
    ctx.send_reply(&packet, response)
}
