use common::time::ServerTime;
use protocol::{
    cs::{
        DcNetWorkingParamEmailsDelete, DcNetWorkingParamEmailsGetEmails,
        DcNetWorkingParamEmailsRead, DcNetWorkingParamEmailsTakeGift, DcNetWorkingResEmailsDelete,
        DcNetWorkingResEmailsGetEmails, DcNetWorkingResEmailsRead, DcNetWorkingResEmailsTakeGift,
    },
    pbcommon::DcNetDataEmailsIDs,
    prost::Message,
};

use crate::net::{context::HandlerContext, error::NetworkResult, packet::ClientPacket};

pub async fn on_list(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamEmailsGetEmails::decode(packet.payload.as_slice())?;
    let emails = ctx.player()?.mails(ServerTime::now_seconds_i32());
    ctx.send_reply(&packet, DcNetWorkingResEmailsGetEmails { emails })
}

pub async fn on_read(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEmailsRead::decode(packet.payload.as_slice())?;
    let ids = request.ids.unwrap_or_default().email_ids;
    let (email_ids, _) = ctx
        .update_player()?
        .read_mails(&ids, ServerTime::now_seconds_i32());
    ctx.send_reply(
        &packet,
        DcNetWorkingResEmailsRead {
            ids: Some(DcNetDataEmailsIDs { email_ids }),
        },
    )
}

pub async fn on_claim_attachments(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamEmailsTakeGift::decode(packet.payload.as_slice())?;
    let ids = request.ids.unwrap_or_default().email_ids;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let now = ServerTime::now_seconds_i32();
    let previous_achievements = ctx.player()?.achievements(tables, now);
    let (rewards, email_ids, _) = ctx
        .update_player()?
        .claim_mail_attachments(&ids, tables, now);
    let achievement_updates =
        ctx.player()?
            .completed_achievements_since(&previous_achievements, tables, now);
    super::achievement::push_updates(ctx, &achievement_updates)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResEmailsTakeGift {
            rewards: Some(rewards),
            email_ids,
        },
    )
}

pub async fn on_delete(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamEmailsDelete::decode(packet.payload.as_slice())?;
    let email_ids = ctx
        .update_player()?
        .delete_mails(&request.ids.unwrap_or_default().email_ids);
    ctx.send_reply(
        &packet,
        DcNetWorkingResEmailsDelete {
            ids: Some(DcNetDataEmailsIDs { email_ids }),
        },
    )
}
