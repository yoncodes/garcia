use protocol::{
    cs::{
        DcNetWorkingParamCollectionList, DcNetWorkingParamCollectionPlace,
        DcNetWorkingParamCollectionPlaceInfos, DcNetWorkingParamCollectionReward,
        DcNetWorkingParamCollectionSuitReward, DcNetWorkingParamCollectionSuitRewardList,
        DcNetWorkingResCollectionList, DcNetWorkingResCollectionPlace,
        DcNetWorkingResCollectionPlaceInfos, DcNetWorkingResCollectionReward,
        DcNetWorkingResCollectionSuitReward, DcNetWorkingResCollectionSuitRewardList,
    },
    prost::Message,
};

use crate::{
    logic::CollectionError,
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamCollectionList::decode(packet.payload.as_slice())?;
    let clist = ctx.player()?.collections.clone();
    ctx.send_reply(&packet, DcNetWorkingResCollectionList { clist })
}

pub async fn on_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCollectionReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let res = ctx
        .update_player()?
        .claim_collection_reward(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_collection)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResCollectionReward {
            id: request.id,
            res: Some(res),
        },
    )
}

pub async fn on_suit_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamCollectionSuitRewardList::decode(packet.payload.as_slice())?;
    let res = ctx.player()?.collection_suit_rewards.clone();
    ctx.send_reply(&packet, DcNetWorkingResCollectionSuitRewardList { res })
}

pub async fn on_suit_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCollectionSuitReward::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (res, has_step) = ctx
        .update_player()?
        .claim_collection_suit_reward(
            request.sid,
            request.step,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_collection)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResCollectionSuitReward {
            res: Some(res),
            has_step,
        },
    )
}

pub async fn on_place(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCollectionPlace::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let place_res = ctx
        .update_player()?
        .place_collection(request.cid, request.pid, tables)
        .map_err(invalid_collection)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResCollectionPlace { code: 0, place_res },
    )
}

pub async fn on_place_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamCollectionPlaceInfos::decode(packet.payload.as_slice())?;
    let res = ctx.player()?.collection_places.clone();
    ctx.send_reply(&packet, DcNetWorkingResCollectionPlaceInfos { res })
}

fn invalid_collection(error: CollectionError) -> NetworkError {
    NetworkError::InvalidCollection(error.to_string())
}
