use protocol::{
    cs::{
        DcNetWorkingParamChallengesList, DcNetWorkingParamChallengesParkourReport,
        DcNetWorkingParamChallengesReport, DcNetWorkingParamInteractObjs,
        DcNetWorkingParamInteracting, DcNetWorkingResChallengesList,
        DcNetWorkingResChallengesParkourReport, DcNetWorkingResChallengesReport,
        DcNetWorkingResInteractObjs, DcNetWorkingResInteracting,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_interact_list(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamInteractObjs::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?.refresh_interactions(tables);
    let objlist = ctx.player()?.interact_objs.clone();
    ctx.send_reply(&request, DcNetWorkingResInteractObjs { objlist })
}

pub async fn on_interact(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamInteracting::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let outcome = ctx
        .update_player()?
        .complete_interaction(&request.object_id, request.status, tables, now)
        .map_err(interaction_error)?;
    let archive_updates =
        ctx.update_player()?
            .newly_unlocked_interaction_archives(&request.object_id, tables, now);
    let task_progress = ctx
        .update_player()?
        .advance_interact_tasks(&request.object_id, request.status, tables, now)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
    super::archive::push_updates(ctx, &archive_updates)?;
    super::tasks::push_progress(ctx, task_progress, now)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResInteracting {
            obj_info: Some(outcome.object),
            reward: outcome.reward,
            remain: Vec::new(),
            follows: Vec::new(),
        },
    )
}

fn interaction_error(error: crate::logic::InteractionError) -> NetworkError {
    match error {
        crate::logic::InteractionError::Unknown(id) => NetworkError::UnknownInteract(id),
        error => NetworkError::InvalidInteraction(error.to_string()),
    }
}

pub async fn on_challenge_list(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamChallengesList::decode(request.payload.as_slice())?;
    let list = ctx.player()?.challenges.clone();
    ctx.send_reply(
        &request,
        DcNetWorkingResChallengesList {
            list,
            point_ids: Vec::new(),
        },
    )
}

pub async fn on_challenge_report(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamChallengesReport::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let (info, reward) = ctx
        .update_player()?
        .complete_city_challenge(
            &request.id,
            0,
            &state.tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidCityChallenge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResChallengesReport {
            info: Some(info),
            reward,
        },
    )
}

pub async fn on_parkour_report(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamChallengesParkourReport::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let (info, reward) = ctx
        .update_player()?
        .complete_city_challenge(
            &request.id,
            request.stars,
            &state.tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidCityChallenge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResChallengesParkourReport {
            info: Some(info),
            reward,
        },
    )
}
