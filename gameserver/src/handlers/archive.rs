use protocol::prost::Message;
use protocol::{
    cs::{
        DcNetWorkingNotifyArchive, DcNetWorkingParamAchiClaim, DcNetWorkingParamAchiList,
        DcNetWorkingParamArchiveList, DcNetWorkingResAchiClaim, DcNetWorkingResAchiList,
        DcNetWorkingResArchiveList,
    },
    pbcommon::{DcNetDataArchiveInfo, Reddot},
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub(crate) fn push_updates(
    ctx: &mut HandlerContext,
    archives: &[DcNetDataArchiveInfo],
) -> NetworkResult<()> {
    for archive in archives {
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyArchive as u16,
            DcNetWorkingNotifyArchive {
                infos: vec![*archive],
            },
        )?;
        super::red_dot::push_one(ctx, Reddot::Archive, archive.id)?;
    }
    Ok(())
}

pub async fn on_achievement_list(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamAchiList::decode(request.payload.as_slice())?;
    let list = ctx.player()?.achievements(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
    );
    ctx.send_reply(&request, DcNetWorkingResAchiList { list })
}

pub async fn on_achievement_claim(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamAchiClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (achi, res) = ctx
        .update_player()?
        .claim_achievement(
            request.id,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(|error| NetworkError::InvalidAchievement(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResAchiClaim {
            res: Some(res),
            achi: Some(achi),
        },
    )
}

pub async fn on_archive_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamArchiveList::decode(request.payload.as_slice())?;
    let infos = ctx.player()?.archive_infos(
        &ctx.state.tables,
        common::time::ServerTime::now_seconds_i32(),
    );
    ctx.send_reply(&request, DcNetWorkingResArchiveList { infos })
}
