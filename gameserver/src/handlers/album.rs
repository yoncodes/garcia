use protocol::{
    cs::{
        DcNetWorkingParamAlbumsList, DcNetWorkingParamAlbumsSelect, DcNetWorkingResAlbumsList,
        DcNetWorkingResAlbumsSelect,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamAlbumsList::decode(request.payload.as_slice())?;
    let albums = ctx.player()?.albums.clone();
    ctx.send_reply(&request, DcNetWorkingResAlbumsList { albums })
}

pub async fn on_select(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamAlbumsSelect::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = common::time::ServerTime::now_seconds_i32();
    let previous_achievements = ctx.player()?.achievements(tables, now);
    ctx.update_player()?
        .select_albums(&request.id_list)
        .map_err(|error| NetworkError::LockedAlbum(error.0))?;
    let achievement_updates =
        ctx.player()?
            .completed_achievements_since(&previous_achievements, tables, now);
    super::achievement::push_updates(ctx, &achievement_updates)?;
    let albums = ctx.player()?.albums.clone();
    ctx.send_reply(&packet, DcNetWorkingResAlbumsSelect { albums })
}
