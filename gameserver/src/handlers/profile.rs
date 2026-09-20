use protocol::{
    cs::{
        DcNetWorkingParamProfileList, DcNetWorkingParamProfileSet, DcNetWorkingResProfileList,
        DcNetWorkingResProfileSet,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamProfileList::decode(request.payload.as_slice())?;
    let player = ctx.player()?;
    let response = DcNetWorkingResProfileList {
        frame_list: player.profile_frames.clone(),
        avatar_list: player.profile_avatars.clone(),
        card_list: player.profile_cards.clone(),
        title_list: player.profile_titles.clone(),
    };
    ctx.send_reply(&request, response)
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamProfileSet::decode(packet.payload.as_slice())?;
    ctx.update_player()?
        .select_profile(request.p_type, request.id)
        .map_err(|_| NetworkError::LockedProfile {
            profile_type: request.p_type,
            id: request.id,
        })?;
    ctx.send_reply(&packet, DcNetWorkingResProfileSet { id: request.id })
}
