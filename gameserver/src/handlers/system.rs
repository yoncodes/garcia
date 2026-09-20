use common::time::ServerTime;
use protocol::{
    cs::{DcNetWorkingParamSysInfo, DcNetWorkingResSysInfo},
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamSysInfo::decode(request.payload.as_slice())?;
    let config = common::config();
    ctx.send_reply(
        &request,
        DcNetWorkingResSysInfo {
            time: ServerTime::now_seconds(),
            zone_offset: config.server.zone_offset,
            assets: config.server.assets_url.clone(),
        },
    )
}
