use protocol::{
    cs::{
        DcNetWorkingParamTrialDone, DcNetWorkingParamTrialInfo, DcNetWorkingParamTrialList,
        DcNetWorkingParamTrialSave, DcNetWorkingResTrialDone, DcNetWorkingResTrialInfo,
        DcNetWorkingResTrialList, DcNetWorkingResTrialSave,
    },
    prost::Message,
};

use crate::{
    logic::TrialDoneOutcome,
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamTrialList::decode(packet.payload.as_slice())?;
    let list = ctx.player()?.trials(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
    );
    ctx.send_reply(&packet, DcNetWorkingResTrialList { list })
}

pub async fn on_save(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTrialSave::decode(packet.payload.as_slice())?;
    let trial_id = request
        .trial
        .ok_or_else(|| NetworkError::InvalidTrial("missing trial".into()))?
        .trial_id;
    let game_role_ids = ctx
        .player()?
        .save_trial(
            trial_id,
            &ctx.state.tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_trial)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTrialSave {
            trial_id,
            game_role_ids,
        },
    )
}

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTrialInfo::decode(packet.payload.as_slice())?;
    let info = ctx
        .player()?
        .trial_info(
            request.trial_id,
            &ctx.state.tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_trial)?;
    ctx.send_reply(&packet, DcNetWorkingResTrialInfo { info: Some(info) })
}

pub async fn on_complete(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamTrialDone::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let TrialDoneOutcome {
        trial_id,
        reward,
        local,
        task,
    } = ctx
        .update_player()?
        .finish_trial(request.trial_id, tables, now)
        .map_err(invalid_trial)?;
    super::tasks::push_task_changes(ctx, &[task])?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTrialDone {
            trial_id,
            reward: Some(reward),
            local,
        },
    )
}

fn invalid_trial(error: crate::logic::TrialError) -> NetworkError {
    NetworkError::InvalidTrial(error.to_string())
}
