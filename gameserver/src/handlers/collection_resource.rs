use protocol::{
    cs::{
        DcNetWorkingParamCollectionResCollect, DcNetWorkingParamCollectionResList,
        DcNetWorkingResCollectionResCollect, DcNetWorkingResCollectionResList,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCollectionResList::decode(packet.payload.as_slice())?;
    let info = ctx
        .player()?
        .collection_resource_info(request.city_id, &ctx.state.tables);
    ctx.send_reply(&packet, DcNetWorkingResCollectionResList { info })
}

pub async fn on_collect(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCollectionResCollect::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let reward = ctx
        .update_player()?
        .collect_collection_resource(
            &request.c_id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidCollectionResource(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResCollectionResCollect {
            reward: Some(reward),
        },
    )
}
