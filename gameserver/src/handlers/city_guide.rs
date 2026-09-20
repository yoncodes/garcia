use protocol::{
    cs::{
        DcNetWorkingParamCityGuideList, DcNetWorkingParamCityGuideSet,
        DcNetWorkingResCityGuideList, DcNetWorkingResCityGuideSet,
    },
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamCityGuideList::decode(request.payload.as_slice())?;
    let ids = ctx.player()?.city_guides.clone();
    ctx.send_reply(&request, DcNetWorkingResCityGuideList { ids })
}

pub async fn on_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamCityGuideSet::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    ctx.update_player()?
        .record_city_guide(request.id.clone(), &state.tables)
        .map_err(|error| NetworkError::UnknownCityGuide(error.0))?;
    let response = DcNetWorkingResCityGuideSet { id: request.id };
    ctx.send_reply(&packet, response)
}
