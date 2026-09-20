use protocol::{
    cs::{
        DcNetWorkingParamMonsterPointHit, DcNetWorkingParamMonsterPoints,
        DcNetWorkingParamMonstersMonsterManual4Port, DcNetWorkingResMonsterPointHit,
        DcNetWorkingResMonsterPoints, DcNetWorkingResMonstersMonsterManual4Port,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_manual_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamMonstersMonsterManual4Port::decode(request.payload.as_slice())?;
    let port_list = ctx.player()?.monster_manuals.clone();
    ctx.send_reply(
        &request,
        DcNetWorkingResMonstersMonsterManual4Port { port_list },
    )
}

pub async fn on_points(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamMonsterPoints::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (groups, _) = ctx.update_player()?.monster_point_groups(
        request.city_id,
        tables,
        common::time::ServerTime::now_seconds_i32(),
    );
    ctx.send_reply(&packet, DcNetWorkingResMonsterPoints { groups })
}

pub async fn on_point_hit(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamMonsterPointHit::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (reward, group, reg_coins) = ctx
        .update_player()?
        .defeat_monster_point(
            &request.point,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidMonsterPoint(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResMonsterPointHit {
            point: request.point,
            reward: Some(reward),
            group: Some(group),
            reg_coins,
        },
    )
}
