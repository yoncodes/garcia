use protocol::{
    cs::{
        DcNetWorkingParamFeatList, DcNetWorkingParamFeatUnlock, DcNetWorkingParamPortList,
        DcNetWorkingParamPortsSett, DcNetWorkingParamPortsStart, DcNetWorkingResFeatList,
        DcNetWorkingResFeatUnlock, DcNetWorkingResInteracting, DcNetWorkingResPortList,
        DcNetWorkingResPortsSett, DcNetWorkingResPortsStart,
    },
    pbcommon::DcNetDataTakeRewardRes,
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_port_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamPortList::decode(request.payload.as_slice())?;
    let ports = ctx.player()?.ports.clone();
    ctx.send_reply(&request, DcNetWorkingResPortList { ports })
}

pub async fn on_feature_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamFeatList::decode(request.payload.as_slice())?;
    let feats = ctx.player()?.feats.clone();
    ctx.send_reply(&request, DcNetWorkingResFeatList { feats })
}

pub async fn on_feature_unlock(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    let parameter = DcNetWorkingParamFeatUnlock::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let feat = ctx
        .update_player()?
        .unlock_feature(parameter.feat_id, tables)
        .map_err(invalid_feature)?;
    ctx.send_reply(&request, DcNetWorkingResFeatUnlock { feat: Some(feat) })
}

fn invalid_feature(error: crate::logic::FeatureError) -> NetworkError {
    NetworkError::InvalidFeature(error.to_string())
}

pub async fn on_port_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPortsStart::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?
        .start_port(request.id, tables)
        .map_err(|error| NetworkError::InvalidPort(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPortsStart {
            id: request.id,
            monster_drop: Vec::new(),
        },
    )
}

pub async fn on_port_settlement(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamPortsSett::decode(packet.payload.as_slice())?;
    let settlement = request
        .sett
        .as_ref()
        .ok_or_else(|| NetworkError::InvalidPort("settlement data is missing".into()))?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (port, roles) = ctx
        .update_player()?
        .settle_port(settlement, tables)
        .map_err(|error| NetworkError::InvalidPort(error.to_string()))?;
    let now = common::time::ServerTime::now_seconds_i32();
    let mut task_progress = ctx
        .update_player()?
        .advance_port_tasks(settlement.id, tables, now)
        .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;

    let interact_obj = if request.object_id.is_empty() {
        None
    } else {
        let obj_info = ctx
            .update_player()?
            .record_interaction(&request.object_id, request.status, tables)
            .map_err(|error| match error {
                crate::logic::InteractionError::Unknown(id) => NetworkError::UnknownInteract(id),
                error => NetworkError::InvalidInteraction(error.to_string()),
            })?;
        task_progress.extend(
            ctx.update_player()?
                .advance_interact_tasks(&request.object_id, request.status, tables, now)
                .map_err(|error| NetworkError::InvalidTask(error.to_string()))?,
        );
        Some(DcNetWorkingResInteracting {
            obj_info,
            reward: None,
            remain: Vec::new(),
            follows: Vec::new(),
        })
    };

    super::tasks::push_progress(ctx, task_progress, now)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResPortsSett {
            curr_gameplay_id: settlement.id,
            take_rewards_res: Some(DcNetDataTakeRewardRes::default()),
            port: Some(port),
            roles,
            user_up_data: None,
            remains: Vec::new(),
            interact_obj,
            boss_drop: None,
            fake_drop_rewards_res: Vec::new(),
            favor_list: Vec::new(),
        },
    )
}
