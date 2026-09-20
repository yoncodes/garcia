use protocol::{
    cs::{
        DcNetWorkingParamDungeonList, DcNetWorkingParamDungeonSett, DcNetWorkingParamDungeonStart,
        DcNetWorkingParamDungeonSweep, DcNetWorkingResDungeonList, DcNetWorkingResDungeonSett,
        DcNetWorkingResDungeonStart, DcNetWorkingResDungeonSweep,
    },
    pbcommon::DcNetDataPhy,
    prost::Message,
};

use crate::{
    logic::{DungeonRuntime, DungeonSettlementOutcome, DungeonSweepOutcome},
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamDungeonList::decode(packet.payload.as_slice())?;
    let ids = ctx.player()?.completed_dungeons.clone();
    ctx.send_reply(&packet, DcNetWorkingResDungeonList { ids })
}

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamDungeonStart::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?
        .start_dungeon(
            request.id,
            request.dungeon_type,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_dungeon)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResDungeonStart {
            id: request.id,
            monster_drop: Vec::new(),
        },
    )
}

pub async fn on_settlement(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamDungeonSett::decode(packet.payload.as_slice())?;
    let settlement = request
        .sett
        .as_ref()
        .ok_or_else(|| NetworkError::InvalidDungeon("settlement data is missing".into()))?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let DungeonSettlementOutcome {
        gameplay_id,
        rewards,
        remains,
        heat_update,
        user_up_data,
        mission_updates,
        achievement_updates,
        archive_updates,
        daily_updates,
        seven_day_updates,
        battle_pass_updates,
    } = ctx
        .update_player()?
        .settle_dungeon(
            settlement,
            request.ntime,
            request.dungeon_type,
            tables,
            DungeonRuntime {
                now,
                zone_offset: common::config().server.zone_offset,
                player_exp_per_stamina: common::config().gameplay.dungeon_player_exp_per_stamina,
            },
        )
        .map_err(invalid_dungeon)?;
    let task_progress = if settlement.is_completed == 0 {
        Vec::new()
    } else {
        ctx.update_player()?
            .advance_dungeon_tasks(settlement.id, tables, now)
            .map_err(|error| NetworkError::InvalidTask(error.to_string()))?
    };
    super::tasks::push_missions(ctx, &mission_updates)?;
    if let Some(update) = &user_up_data {
        let mut notification = update.clone();
        // Live cmd 10020 dungeon notifications set this refresh flag even when the
        // response correctly reports that the player did not gain a level.
        notification.is_lv_up = true;
        ctx.push(
            protocol::proids::NotifyId::DcNetDataAttrUpData as u16,
            notification,
        )?;
    }
    super::battle_pass::push_task_updates(ctx, &battle_pass_updates)?;
    if let Some(update) = heat_update {
        ctx.push(
            protocol::proids::NotifyId::DcNetDataPhy as u16,
            DcNetDataPhy {
                item: Some(update.item),
                cd_time: update.cd_time,
            },
        )?;
    }
    super::achievement::push_updates(ctx, &achievement_updates)?;
    super::archive::push_updates(ctx, &archive_updates)?;
    super::tasks::push_daily_updates(ctx, &daily_updates)?;
    super::activity::push_seven_day_updates(ctx, &seven_day_updates)?;
    super::tasks::push_progress(ctx, task_progress, now)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResDungeonSett {
            curr_gameplay_id: gameplay_id,
            take_rewards_res: Some(rewards),
            remains,
            user_up_data,
            ..Default::default()
        },
    )
}

pub async fn on_sweep(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamDungeonSweep::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let DungeonSweepOutcome {
        infos,
        remains,
        heat_update,
        archive_updates,
    } = ctx
        .update_player()?
        .sweep_dungeon(
            request.id,
            request.num,
            tables,
            common::time::ServerTime::now_seconds_i32(),
            common::config().gameplay.dungeon_player_exp_per_stamina,
        )
        .map_err(invalid_dungeon)?;
    if let Some(update) = heat_update {
        ctx.push(
            protocol::proids::NotifyId::DcNetDataPhy as u16,
            DcNetDataPhy {
                item: Some(update.item),
                cd_time: update.cd_time,
            },
        )?;
    }
    super::archive::push_updates(ctx, &archive_updates)?;
    ctx.send_reply(&packet, DcNetWorkingResDungeonSweep { infos, remains })
}

fn invalid_dungeon(error: crate::logic::DungeonError) -> NetworkError {
    NetworkError::InvalidDungeon(error.to_string())
}
