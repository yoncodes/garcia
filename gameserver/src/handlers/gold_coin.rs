use protocol::{
    cs::{
        DcNetWorkingParamGoldCoinCollect, DcNetWorkingParamGoldCoinList,
        DcNetWorkingResGoldCoinCollect, DcNetWorkingResGoldCoinList,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamGoldCoinList::decode(packet.payload.as_slice())?;
    let mut coid_ids = vec![String::new()];
    coid_ids.extend(ctx.player()?.gold_coin_ids(request.city_id));
    ctx.send_reply(&packet, DcNetWorkingResGoldCoinList { coid_ids })
}

pub async fn on_collect(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamGoldCoinCollect::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let reward = ctx
        .update_player()?
        .collect_gold_coin(request.city_id, &request.coin_id, tables)
        .map_err(|error| NetworkError::InvalidGoldCoin(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResGoldCoinCollect {
            reward: Some(reward),
        },
    )
}
