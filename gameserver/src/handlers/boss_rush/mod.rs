use configs::GameTables;
use database::models::game::boss_rush::BossRushOverride;
use protocol::{
    cs::{
        DcNetWorkingNotifyEmails, DcNetWorkingParamBossRushInfo, DcNetWorkingParamBossRushRank,
        DcNetWorkingParamBossRushResetTeam, DcNetWorkingParamBossRushReward,
        DcNetWorkingParamBossRushSett, DcNetWorkingResBossRushInfo, DcNetWorkingResBossRushRank,
        DcNetWorkingResBossRushResetTeam, DcNetWorkingResBossRushReward,
        DcNetWorkingResBossRushSett,
    },
    pbcommon::{DcNetDataBossRushInfo, DcNetDataBossRushRankElem},
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamBossRushInfo::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let override_period = boss_rush_override(ctx).await?;
    let mut response = boss_rush_info(
        &ctx.state.tables,
        now,
        common::config().server.zone_offset,
        override_period,
    );
    let bid = response.boss_info.as_ref().map_or(0, |info| {
        if info.curr_bid == 0 {
            info.last_bid
        } else {
            info.curr_bid
        }
    });
    let (boss_base, ids, max_damage, ranking_mail) = {
        let state = ctx.state.clone();
        let tables = &state.tables;
        let player = ctx.update_player()?;
        let ranking_mail = ranking_reward_ready_at(
            tables,
            player.boss_rush.id,
            common::config().server.zone_offset,
            override_period,
        )
        .filter(|ready_at| now >= *ready_at)
        .and_then(|_| {
            player.deliver_boss_rush_ranking_reward(player.boss_rush.id, 1, 1, tables, now)
        });
        if bid != 0 {
            player.sync_boss_rush(bid);
        }
        (
            player.boss_rush.ports.clone(),
            player.boss_rush.rewards.clone(),
            player.boss_rush_score(),
            ranking_mail,
        )
    };
    if let Some(info) = response.boss_info.as_mut() {
        info.boss_base = boss_base;
        info.ids = ids;
        info.max_damage = max_damage;
    }
    if let Some(mail) = ranking_mail {
        tracing::info!(
            bid,
            rank = 1,
            reward_count = mail.gift_list.len(),
            "boss rush ranking reward delivered by mail"
        );
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyEmails as u16,
            DcNetWorkingNotifyEmails { emails: vec![mail] },
        )?;
    }
    ctx.send_reply(&packet, response)
}

pub async fn on_rank(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBossRushRank::decode(packet.payload.as_slice())?;
    let player = ctx.player()?;
    let score = player.boss_rush_score();
    let ranked = request.bid == player.boss_rush.id && score > 0;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBossRushRank {
            ranks: ranked
                .then(|| DcNetDataBossRushRankElem {
                    rank: 1,
                    score: score as f64,
                    name: player.nickname.clone(),
                    rids: player.boss_rush.ports.clone(),
                    avatar: player.profile_avatar,
                    avatar_frame: player.profile_frame,
                    title: player.profile_title,
                    uid: player.uid,
                    level: player.level,
                })
                .into_iter()
                .collect(),
            cur_rank: u32::from(ranked),
            all_rank_num: u32::from(ranked),
        },
    )
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBossRushSett::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let override_period = boss_rush_override(ctx).await?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event = boss_rush_runtime(
        tables,
        now,
        common::config().server.zone_offset,
        override_period,
    )
    .map(|(event, _)| event)
    .ok_or_else(|| NetworkError::InvalidBossRush("no active event".into()))?;
    let (score, boss_base) = ctx
        .update_player()?
        .settle_boss_rush(event, request.pid, request.damage, request.role_ids)
        .map_err(|error| NetworkError::InvalidBossRush(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBossRushSett {
            pid: request.pid,
            score,
            boss_base,
        },
    )
}

pub async fn on_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBossRushReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (take_res, sids) = ctx
        .update_player()?
        .claim_boss_rush_rewards(
            request.bid,
            request.sid,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidBossRush(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResBossRushReward {
            take_res: Some(take_res),
            sids,
        },
    )
}

pub async fn on_reset_team(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamBossRushResetTeam::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let override_period = boss_rush_override(ctx).await?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let event = boss_rush_runtime(
        tables,
        now,
        common::config().server.zone_offset,
        override_period,
    )
    .map(|(event, _)| event)
    .ok_or_else(|| NetworkError::InvalidBossRush("no active event".into()))?;
    let boss_base = ctx
        .update_player()?
        .reset_boss_rush_team(event, request.pid)
        .map_err(|error| NetworkError::InvalidBossRush(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResBossRushResetTeam { boss_base })
}

fn boss_rush_info(
    tables: &GameTables,
    now: i32,
    zone_offset: i32,
    override_period: Option<BossRushOverride>,
) -> DcNetWorkingResBossRushInfo {
    let timestamp = i64::from(now);
    let active = boss_rush_runtime(tables, now, zone_offset, override_period);
    let table_last = tables
        .boss_rushes
        .rows
        .iter()
        .filter(|event| {
            common::time::table_time_utc(&event.open_time, zone_offset)
                .is_some_and(|open| open <= timestamp)
        })
        .max_by_key(|event| event.order);
    let last = override_period
        .filter(|entry| entry.expires_at <= now)
        .and_then(|entry| tables.boss_rushes.get(entry.event_id))
        .or(table_last);
    let next_open = tables
        .boss_rushes
        .rows
        .iter()
        .filter_map(|event| common::time::table_time_utc(&event.open_time, zone_offset))
        .filter(|open| *open > timestamp)
        .min();
    let close = active.map_or_else(
        || {
            override_period
                .filter(|entry| entry.expires_at <= now)
                .map_or_else(
                    || {
                        last.and_then(|event| {
                            common::time::table_time_utc(&event.close_time, zone_offset)
                        })
                        .unwrap_or_default()
                    },
                    |entry| i64::from(entry.expires_at),
                )
        },
        |(_, close)| close,
    );

    DcNetWorkingResBossRushInfo {
        open_state: active.is_some(),
        countdown: active.map_or(0, |_| common::time::seconds_until(close, timestamp)),
        boss_info: Some(DcNetDataBossRushInfo {
            boss_base: Vec::new(),
            ids: Vec::new(),
            curr_bid: active.map_or(0, |(event, _)| event.id),
            last_bid: last.map_or(0, |event| event.id),
            max_damage: 0,
        }),
        next_open_countdown: next_open
            .map_or(0, |open| common::time::seconds_until(open, timestamp)),
        close_time: close,
    }
}

fn boss_rush_runtime(
    tables: &GameTables,
    now: i32,
    zone_offset: i32,
    override_period: Option<BossRushOverride>,
) -> Option<(&configs::tables::BossRush, i64)> {
    active_boss_rush(tables, now, zone_offset)
        .and_then(|event| {
            common::time::table_time_utc(&event.close_time, zone_offset).map(|close| (event, close))
        })
        .or_else(|| {
            let entry = override_period.filter(|entry| entry.expires_at > now)?;
            tables
                .boss_rushes
                .get(entry.event_id)
                .map(|event| (event, i64::from(entry.expires_at)))
        })
}

async fn boss_rush_override(ctx: &HandlerContext) -> NetworkResult<Option<BossRushOverride>> {
    Ok(database::db::boss_rush_override::current(&ctx.state.db).await?)
}

fn ranking_reward_ready_at(
    tables: &GameTables,
    bid: i32,
    zone_offset: i32,
    override_period: Option<BossRushOverride>,
) -> Option<i32> {
    let (close, instant_rewards) = override_period
        .filter(|entry| entry.event_id == bid)
        .map(|entry| (i64::from(entry.expires_at), entry.instant_rewards))
        .or_else(|| {
            let event = tables.boss_rushes.get(bid)?;
            common::time::table_time_utc(&event.close_time, zone_offset).map(|close| (close, false))
        })?;
    let delay = if instant_rewards {
        0
    } else {
        i64::from(tables.cultivation_constants.boss_rush_reward_delay_hours) * 60 * 60
    };
    i32::try_from(close.checked_add(delay)?).ok()
}

fn active_boss_rush(
    tables: &GameTables,
    now: i32,
    zone_offset: i32,
) -> Option<&configs::tables::BossRush> {
    tables.boss_rushes.rows.iter().find(|event| {
        common::time::table_window_active(&event.open_time, &event.close_time, now, zone_offset)
    })
}

#[cfg(test)]
mod tests;
