use protocol::{
    cs::{DcNetWorkingParamRoleSkinGetList, DcNetWorkingResRoleSkinGetList},
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_skin_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRoleSkinGetList::decode(request.payload.as_slice())?;
    let skins = ctx.player()?.skins.clone();
    ctx.send_reply(&request, DcNetWorkingResRoleSkinGetList { skins })
}
