use protocol::{
    cs::{
        DcNetWorkingParamPlayingPortGet, DcNetWorkingParamPlayingPortSet,
        DcNetWorkingResPlayingPortGet, DcNetWorkingResPlayingPortSet,
    },
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamPlayingPortGet::decode(packet.payload.as_slice())?;
    let (port_index_id, port_info) = ctx
        .player()?
        .playing_port
        .as_ref()
        .map_or((0, String::new()), |port| {
            (port.port_index_id, port.port_info.clone())
        });
    ctx.send_reply(
        &packet,
        DcNetWorkingResPlayingPortGet {
            port_index_id,
            port_info,
            monster_drop: Vec::new(),
        },
    )
}

pub async fn on_update(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamPlayingPortSet::decode(packet.payload.as_slice())?;
    ctx.update_player()?
        .set_playing_port(request.port_index_id, request.port_info);
    ctx.send_reply(&packet, DcNetWorkingResPlayingPortSet { code: 0 })
}
