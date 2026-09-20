use protocol::{
    cs::{
        DcNetWorkingNotifyProfileNew, DcNetWorkingParamGachaClaimCumReward,
        DcNetWorkingParamGachaClaimExtra, DcNetWorkingParamGachaDo, DcNetWorkingParamGachaGetList,
        DcNetWorkingParamGachaLogs, DcNetWorkingResGachaClaimCumReward, DcNetWorkingResGachaDo,
        DcNetWorkingResGachaGetList, DcNetWorkingResGachaLogs,
    },
    pbcommon::Reddot,
    prost::Message,
};

use crate::logic::GachaEvent;
use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamGachaGetList::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let banner_overrides = database::db::gacha_banner_overrides::active(&ctx.state.db, now)
        .await?
        .into_iter()
        .map(|banner| (banner.gacha_id, banner.expires_at))
        .collect::<Vec<_>>();
    let gacha_list = ctx.player()?.gacha_pools_with_overrides(
        &ctx.state.tables,
        now,
        common::config().server.zone_offset,
        &banner_overrides,
    );
    ctx.send_reply(&packet, DcNetWorkingResGachaGetList { gacha_list })
}

pub async fn on_draw(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamGachaDo::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let override_expires_at = database::db::gacha_banner_overrides::active(&state.db, now)
        .await?
        .into_iter()
        .find_map(|banner| (banner.gacha_id == request.gacha_id).then_some(banner.expires_at));
    let outcome = ctx
        .update_player()?
        .draw_gacha_with_override(
            request.gacha_id,
            request.gacha_type,
            tables,
            now,
            common::config().server.zone_offset,
            override_expires_at,
        )
        .map_err(|error| NetworkError::InvalidGacha(error.to_string()))?;
    push_events(ctx, &outcome.events)?;
    super::battle_pass::push_task_updates(ctx, &outcome.battle_pass_updates)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResGachaDo {
            reward_list: outcome.reward_list,
            gacha_data: Some(outcome.gacha_data),
            remains: outcome.remains,
            coin: outcome.coin,
        },
    )
}

pub async fn on_history(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamGachaLogs::decode(packet.payload.as_slice())?;
    let logs = ctx
        .player()?
        .gacha_logs(request.group_id, &ctx.state.tables);
    ctx.send_reply(&packet, DcNetWorkingResGachaLogs { logs })
}

pub async fn on_claim_cumulative(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamGachaClaimCumReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let outcome = ctx
        .update_player()?
        .claim_gacha_cumulative(
            request.reward_id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidGacha(error.to_string()))?;
    push_events(ctx, &outcome.events)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResGachaClaimCumReward {
            reward: Some(outcome.reward),
            gacha_data: Some(outcome.gacha_data),
        },
    )
}

fn push_events(ctx: &mut HandlerContext, events: &[GachaEvent]) -> NetworkResult<()> {
    for event in events {
        match event {
            GachaEvent::Achievement(achievement) => {
                super::achievement::push_updates(ctx, &[*achievement])?;
            }
            GachaEvent::Archive(archive) => {
                super::archive::push_updates(ctx, &[*archive])?;
            }
            GachaEvent::ProfileAvatar(avatar) => {
                super::red_dot::push_one(ctx, Reddot::Avatar, avatar.id)?;
                ctx.push(
                    protocol::proids::NotifyId::DcNetWorkingNotifyProfileNew as u16,
                    DcNetWorkingNotifyProfileNew {
                        avatar: Some(*avatar),
                        card: None,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub async fn on_claim_extra(_ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamGachaClaimExtra::decode(packet.payload.as_slice())?;
    Err(NetworkError::InvalidGacha(format!(
        "extra-reward entitlement is not recovered (coin_or_role={})",
        request.coin_or_role
    )))
}
