use protocol::{
    cs::{
        DcNetWorkingParamRoleAttrList, DcNetWorkingParamRoleAttrSave, DcNetWorkingResRoleAttrList,
        DcNetWorkingResRoleAttrSave,
    },
    prost::Message,
};

use crate::{
    logic::RoleAttrError,
    net::{
        context::HandlerContext,
        error::{NetworkError, NetworkResult},
        packet::ClientPacket,
    },
};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRoleAttrList::decode(packet.payload.as_slice())?;
    let role_infos = ctx.player()?.role_attrs.clone();
    ctx.send_reply(&packet, DcNetWorkingResRoleAttrList { role_infos })
}

pub async fn on_save(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRoleAttrSave::decode(packet.payload.as_slice())?;
    ctx.update_player()?
        .save_role_attrs(request.role_infos)
        .map_err(invalid_role_attr)?;
    ctx.send_reply(&packet, DcNetWorkingResRoleAttrSave { state: true })
}

fn invalid_role_attr(error: RoleAttrError) -> NetworkError {
    NetworkError::InvalidRoleAttr(error.to_string())
}
