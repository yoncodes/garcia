use protocol::{
    cs::{
        DcNetWorkingParamSmsList, DcNetWorkingParamSmsRead, DcNetWorkingResSmsList,
        DcNetWorkingResSmsRead,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamSmsList::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    ctx.update_player()?.sync_sms(tables);
    let list = ctx.player()?.sms.clone();
    ctx.send_reply(&packet, DcNetWorkingResSmsList { list })
}

pub async fn on_read(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamSmsRead::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (res, _) = ctx
        .update_player()?
        .read_sms(request.id, request.selected, tables)
        .map_err(|error| NetworkError::InvalidSms(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResSmsRead { res: Some(res) })
}
