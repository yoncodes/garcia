use common::{config, time::ServerTime};
use protocol::{
    cs::{
        DcNetWorkingNotifyReddots, DcNetWorkingParamReddotCheck, DcNetWorkingParamReddotList,
        DcNetWorkingResReddotCheck, DcNetWorkingResReddotList,
    },
    pbcommon::{DcNetDataRedDot, Reddot},
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub(super) fn push_each(
    ctx: &mut HandlerContext,
    red_dots: Vec<DcNetDataRedDot>,
) -> NetworkResult<()> {
    for red_dot in red_dots {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyReddots as u16,
            DcNetWorkingNotifyReddots {
                reddots: vec![red_dot],
            },
        )?;
    }
    Ok(())
}

pub(crate) fn push_one(
    ctx: &mut HandlerContext,
    kind: Reddot,
    value: impl ToString,
) -> NetworkResult<()> {
    let arg = value.to_string();
    push_each(
        ctx,
        vec![DcNetDataRedDot {
            id: format!("{}_{}", kind as i32, arg),
            func_id: kind as i32,
            arg,
            ..Default::default()
        }],
    )
}

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamReddotList::decode(packet.payload.as_slice())?;
    let reddots = ctx.player()?.red_dots(
        &ctx.state.tables,
        ServerTime::now_seconds_i32(),
        config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResReddotList { reddots })
}

pub async fn on_check(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamReddotCheck::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (reddots, _) = ctx.update_player()?.check_red_dots(
        &request.ids,
        tables,
        ServerTime::now_seconds_i32(),
        config().server.zone_offset,
    );
    ctx.send_reply(&packet, DcNetWorkingResReddotCheck { reddots })
}
