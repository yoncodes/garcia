use protocol::{
    cs::{
        DcNetWorkingParamUserGuideList, DcNetWorkingParamUserGuideSave,
        DcNetWorkingResUserGuideList, DcNetWorkingResUserGuideSave,
    },
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

/// Custom broadcast registered by Garcia's HybridCLR client patch.
pub(crate) const TUTORIAL_REFRESH_NOTIFY_ID: u16 = 10_078;

pub(crate) fn push_list(ctx: &mut HandlerContext) -> NetworkResult<()> {
    let gids = ctx.player()?.user_guides.clone();
    ctx.push(
        TUTORIAL_REFRESH_NOTIFY_ID,
        DcNetWorkingResUserGuideList { gids },
    )
}

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamUserGuideList::decode(request.payload.as_slice())?;
    let gids = ctx.player()?.user_guides.clone();
    ctx.send_reply(&request, DcNetWorkingResUserGuideList { gids })
}

pub async fn on_save(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamUserGuideSave::decode(packet.payload.as_slice())?;
    ctx.update_player()?.record_user_guide(request.gid);
    // Live successful saves return 1 (capture sequences 288, 333, and 1072).
    ctx.send_reply(&packet, DcNetWorkingResUserGuideSave { code: 1 })
}
