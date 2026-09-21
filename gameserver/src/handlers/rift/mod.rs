use configs::{GameTables, tables::Rift};
use protocol::{
    cs::{
        DcNetWorkingParamRiftDefeat, DcNetWorkingParamRiftInfo, DcNetWorkingParamRiftNext,
        DcNetWorkingParamRiftRank, DcNetWorkingParamRiftReport, DcNetWorkingParamRiftSettlment,
        DcNetWorkingParamRiftStart, DcNetWorkingParamRiftTask, DcNetWorkingParamRiftTaskClaim,
        DcNetWorkingResRiftDefeat, DcNetWorkingResRiftInfo, DcNetWorkingResRiftNext,
        DcNetWorkingResRiftRank, DcNetWorkingResRiftReport, DcNetWorkingResRiftSettlment,
        DcNetWorkingResRiftStart, DcNetWorkingResRiftTask, DcNetWorkingResRiftTaskClaim,
    },
    pbcommon::{DcNetDataRift, DcNetDataRiftRank, DcNetDataRiftTask},
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRiftInfo::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let zone_offset = common::config().server.zone_offset;
    let state = ctx.state.clone();
    let rift = current_rift(&state.tables, now, zone_offset);
    let my_rank = {
        let mut player = ctx.update_player()?;
        if let Some(rift) = rift {
            player.sync_rift(rift.id);
        }
        player_rift_rank(&player)
    };
    let mut info = rift_info(&ctx.state.tables, now, zone_offset);
    if let Some(info) = info.as_mut() {
        let player = ctx.player()?;
        info.best_buff_score = player.rift.best_buff_score.into();
        // The live Rift_Info response sends the current ticket item through
        // `rift_initail_item`; the client applies it to its item cache. Garcia
        // grants the table-defined initial tickets when the player is created,
        // so report that persisted item here without granting it a second time.
        info.rift_initail_item = player
            .items
            .iter()
            .find(|item| item.item_id == state.tables.cultivation_constants.ticket_item_id)
            .copied();
    }
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftInfo {
            rift: info,
            my_rank: Some(my_rank),
            defeat: None,
        },
    )
}

pub async fn on_rank(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRiftRank::decode(packet.payload.as_slice())?;
    let rank = player_rift_rank(ctx.player()?);
    let ranked = rank.score > 0;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftRank {
            ranks: ranked.then(|| rank.clone()).into_iter().collect(),
            my_rank: Some(rank),
            total: i32::from(ranked),
        },
    )
}

pub async fn on_task(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRiftTask::decode(packet.payload.as_slice())?;
    let rift = current_rift(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    );
    let player = ctx.player()?;
    let tasks = rift.map_or_else(Vec::new, |rift| {
        rift_tasks(&ctx.state.tables, rift, Some(&player.rift))
    });
    ctx.send_reply(&packet, DcNetWorkingResRiftTask { tasks })
}

pub async fn on_task_claim(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRiftTaskClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event = current_rift(
        tables,
        common::time::ServerTime::now_seconds_i32(),
        common::config().server.zone_offset,
    )
    .ok_or_else(|| NetworkError::InvalidRift("no current event".into()))?;
    let reward = ctx
        .update_player()?
        .claim_rift_task(
            event,
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidRift(error.to_string()))?;
    let player = ctx.player()?;
    let task = tables
        .rift_rewards
        .iter()
        .find(|task| task.id == request.id && task.group == event.reward_group)
        .map(|task| rift_task(task, Some(&player.rift)));
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftTaskClaim {
            id: request.id,
            reward: Some(reward),
            task,
        },
    )
}

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRiftStart::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event =
        active_rift(tables).ok_or_else(|| NetworkError::InvalidRift("no active event".into()))?;
    let (port_id, remain) = ctx
        .update_player()?
        .start_rift(event, &request.buff_map, request.roles, tables)
        .map_err(|error| NetworkError::InvalidRift(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftStart {
            id: event.id,
            remain,
            rift_port_id: port_id,
        },
    )
}

pub async fn on_next(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRiftNext::decode(packet.payload.as_slice())?;
    let player = ctx.player()?;
    if !player.rift.active {
        return Err(NetworkError::InvalidRift("no active run".into()));
    }
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftNext {
            rift_port_id: player.rift.current_port_id,
        },
    )
}

pub async fn on_report(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRiftReport::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event =
        active_rift(tables).ok_or_else(|| NetworkError::InvalidRift("no active event".into()))?;
    let (duration, next_port_id) = ctx
        .update_player()?
        .report_rift_stage(event, request.dur_time, tables)
        .map_err(|error| NetworkError::InvalidRift(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftReport {
            dur_time: duration,
            next_rift_port_id: next_port_id,
        },
    )
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRiftSettlment::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event =
        active_rift(tables).ok_or_else(|| NetworkError::InvalidRift("no active event".into()))?;
    let (duration, score, port_id, buff_score) = ctx
        .update_player()?
        .settle_rift(event, request.dur_time, tables)
        .map_err(|error| NetworkError::InvalidRift(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftSettlment {
            dur_time: duration,
            rank: i32::from(score > 0),
            score,
            rift_port_id: port_id,
            buff_score,
        },
    )
}

pub async fn on_defeat(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRiftDefeat::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let event = active_rift(&state.tables)
        .ok_or_else(|| NetworkError::InvalidRift("no active event".into()))?;
    let score = ctx
        .update_player()?
        .defeat_rift(event.id)
        .map_err(|error| NetworkError::InvalidRift(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRiftDefeat {
            rank: i32::from(score > 0),
            score,
        },
    )
}

fn active_rift(tables: &GameTables) -> Option<&Rift> {
    let now = common::time::ServerTime::now_seconds_i32();
    let event = current_rift(tables, now, common::config().server.zone_offset)?;
    common::time::table_time_utc(&event.close_time, common::config().server.zone_offset)
        .is_some_and(|close| i64::from(now) < close)
        .then_some(event)
}

fn player_rift_rank(player: &crate::logic::Player) -> DcNetDataRiftRank {
    let ranked = player.rift.best_score > 0;
    DcNetDataRiftRank {
        score: player.rift.best_score,
        rank: i32::from(ranked),
        nickname: player.nickname.clone(),
        roles: player.rift.roles.clone(),
        avatar: player.profile_avatar,
        avatar_frame: player.profile_frame,
        title: player.profile_title,
        uid: player.uid,
        buff_score: player.rift.best_buff_score.into(),
        level: player.level,
    }
}

fn current_rift(tables: &GameTables, now: i32, zone_offset: i32) -> Option<&Rift> {
    tables
        .rifts
        .rows
        .iter()
        .filter(|rift| {
            common::time::table_time_utc(&rift.open_time, zone_offset)
                .is_some_and(|open| open <= i64::from(now))
        })
        .max_by_key(|rift| rift.id)
        .filter(|rift| {
            next_rift(tables, rift)
                .and_then(|next| common::time::table_time_utc(&next.open_time, zone_offset))
                .is_none_or(|next_open| i64::from(now) < next_open)
        })
}

fn next_rift<'a>(tables: &'a GameTables, current: &Rift) -> Option<&'a Rift> {
    tables
        .rifts
        .rows
        .iter()
        .filter(|rift| rift.id > current.id)
        .min_by_key(|rift| rift.id)
}

fn rift_info(tables: &GameTables, now: i32, zone_offset: i32) -> Option<DcNetDataRift> {
    let rift = current_rift(tables, now, zone_offset)?;
    let battle_end = common::time::table_time_utc(&rift.close_time, zone_offset)?;
    let rest_end = next_rift(tables, rift)
        .and_then(|next| common::time::table_time_utc(&next.open_time, zone_offset))
        .unwrap_or(battle_end);
    Some(DcNetDataRift {
        id: rift.id,
        battle_remain_sec: common::time::seconds_until(battle_end, i64::from(now)),
        rest_remain_sec: common::time::seconds_until(rest_end, i64::from(now)),
        rift_initail_item: None,
        daily_took_item: None,
        battle_end_time: battle_end,
        rest_end_time: rest_end,
        best_buff_score: 0,
    })
}

fn rift_tasks(
    tables: &GameTables,
    rift: &Rift,
    state: Option<&crate::logic::RiftState>,
) -> Vec<DcNetDataRiftTask> {
    let mut rows: Vec<_> = tables
        .rift_rewards
        .iter()
        .filter(|reward| reward.group == rift.reward_group)
        .collect();
    rows.sort_by_key(|reward| reward.sort);
    rows.into_iter().map(|row| rift_task(row, state)).collect()
}

fn rift_task(
    row: &configs::tables::RiftReward,
    state: Option<&crate::logic::RiftState>,
) -> DcNetDataRiftTask {
    DcNetDataRiftTask {
        id: row.id,
        progress: row.finish_limit.first().map_or(0, |limit| {
            state.map_or(0, |state| state.task_progress(limit.key))
        }),
        total: row
            .finish_limit
            .first()
            .and_then(|limit| limit.value.parse().ok())
            .unwrap_or_default(),
        took: state.is_some_and(|state| state.tasks.contains(&row.id)),
        reward: row
            .reward
            .iter()
            .map(|reward| (reward.key, reward.value))
            .collect(),
    }
}

#[cfg(test)]
mod tests;
