use protocol::{
    cs::{
        DcNetWorkingParamFriendGiftClaim, DcNetWorkingParamFriendGiftSend,
        DcNetWorkingParamFriendsApprove, DcNetWorkingParamFriendsDelete,
        DcNetWorkingParamFriendsInfo, DcNetWorkingParamFriendsProfile,
        DcNetWorkingParamFriendsRequest, DcNetWorkingParamFriendsSearch,
        DcNetWorkingResFriendGiftClaim, DcNetWorkingResFriendGiftSend,
        DcNetWorkingResFriendsApprove, DcNetWorkingResFriendsDelete, DcNetWorkingResFriendsInfo,
        DcNetWorkingResFriendsProfile, DcNetWorkingResFriendsRequest, DcNetWorkingResFriendsSearch,
    },
    pbcommon::{DcNetDataFriendsInfo, DcNetDataSocialUserProfile},
    prost::Message,
};

use crate::{
    logic::Player,
    net::{
        context::HandlerContext, error::NetworkError, error::NetworkResult, packet::ClientPacket,
    },
};

pub async fn on_info(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamFriendsInfo::decode(packet.payload.as_slice())?;
    let (friends_info, gift_num) = friends_info(ctx).await?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFriendsInfo {
            friends_info: Some(friends_info),
            gift_num,
        },
    )
}

pub async fn on_search(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendsSearch::decode(packet.payload.as_slice())?;
    let profile = load_profile(ctx, request.fuid, false, false).await?;
    ctx.send_reply(&packet, DcNetWorkingResFriendsSearch { profile })
}

pub async fn on_request(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendsRequest::decode(packet.payload.as_slice())?;
    let uid = ctx.player()?.uid;
    let state = social_state(ctx, uid).await?;
    if state.friends.len()
        >= usize::try_from(ctx.state.tables.cultivation_constants.friend_maximum).unwrap_or(0)
    {
        return Err(invalid("friend limit reached"));
    }
    if !database::db::social::send_friend_request(
        &ctx.state.db,
        uid,
        request.fuid,
        common::time::ServerTime::now_seconds_i32(),
    )
    .await?
    {
        return Err(invalid(format!("cannot request player {}", request.fuid)));
    }
    ctx.send_reply(
        &packet,
        DcNetWorkingResFriendsRequest { fuid: request.fuid },
    )
}

pub async fn on_approve(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendsApprove::decode(packet.payload.as_slice())?;
    let uid = ctx.player()?.uid;
    if !database::db::social::accept_friend_request(
        &ctx.state.db,
        uid,
        request.fuid,
        ctx.state.tables.cultivation_constants.friend_maximum,
        common::time::ServerTime::now_seconds_i32(),
    )
    .await?
    {
        return Err(invalid(format!("cannot approve player {}", request.fuid)));
    }
    let (friends_info, _) = friends_info(ctx).await?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFriendsApprove {
            friends_info: Some(friends_info),
        },
    )
}

pub async fn on_delete(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendsDelete::decode(packet.payload.as_slice())?;
    let uid = ctx.player()?.uid;
    if !database::db::social::remove_friend(&ctx.state.db, uid, request.fuid).await? {
        return Err(invalid(format!("player {} is not a friend", request.fuid)));
    }
    let (friends_info, _) = friends_info(ctx).await?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResFriendsDelete {
            friends_info: Some(friends_info),
        },
    )
}

pub async fn on_send_gift(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendGiftSend::decode(packet.payload.as_slice())?;
    let uid = ctx.player()?.uid;
    if !database::db::social::send_gift(
        &ctx.state.db,
        uid,
        request.fuid,
        common::time::day_index(common::time::ServerTime::now_seconds_i32()),
    )
    .await?
    {
        return Err(invalid(format!(
            "cannot send gift to player {}",
            request.fuid
        )));
    }
    let friends = load_friend_profile(ctx, uid, request.fuid)
        .await?
        .into_iter()
        .collect();
    ctx.send_reply(&packet, DcNetWorkingResFriendGiftSend { friends })
}

pub async fn on_claim_gift(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendGiftClaim::decode(packet.payload.as_slice())?;
    let uid = ctx.player()?.uid;
    let gift_num = database::db::social::claim_gift(
        &ctx.state.db,
        uid,
        request.fuid,
        common::time::day_index(common::time::ServerTime::now_seconds_i32()),
        ctx.state.tables.cultivation_constants.friend_gift_maximum,
    )
    .await?
    .ok_or_else(|| invalid(format!("cannot claim gift from player {}", request.fuid)))?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let gifts = ctx.update_player()?.grant_friend_gift(tables);
    let friends = load_friend_profile(ctx, uid, request.fuid)
        .await?
        .into_iter()
        .collect();
    ctx.send_reply(
        &packet,
        DcNetWorkingResFriendGiftClaim {
            friends,
            gift_num,
            gifts,
        },
    )
}

pub async fn on_profile(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamFriendsProfile::decode(packet.payload.as_slice())?;
    let profile = load_player(ctx, request.fuid).await?;
    let response = profile.map_or_else(DcNetWorkingResFriendsProfile::default, |player| {
        DcNetWorkingResFriendsProfile {
            profile: Some(player.social_profile(
                &ctx.state.tables,
                false,
                false,
                common::time::ServerTime::now_seconds_i32(),
            )),
            rift_rank: Some(protocol::pbcommon::DcNetDataRiftRank {
                score: player.rift.best_score,
                nickname: player.nickname.clone(),
                roles: player.rift.roles.clone(),
                avatar: player.profile_avatar,
                avatar_frame: player.profile_frame,
                title: player.profile_title,
                uid: player.uid,
                buff_score: i64::from(player.rift.best_buff_score),
                level: player.level,
                ..Default::default()
            }),
            boss_rush_rank: Some(protocol::pbcommon::DcNetDataBossRushRankElem {
                score: player.boss_rush_score() as f64,
                name: player.nickname.clone(),
                rids: player.boss_rush.ports.clone(),
                avatar: player.profile_avatar,
                avatar_frame: player.profile_frame,
                title: player.profile_title,
                uid: player.uid,
                level: player.level,
                ..Default::default()
            }),
            ..Default::default()
        }
    });
    ctx.send_reply(&packet, response)
}

async fn friends_info(ctx: &HandlerContext) -> NetworkResult<(DcNetDataFriendsInfo, i32)> {
    let uid = ctx.player()?.uid;
    let state = social_state(ctx, uid).await?;
    let mut friends = Vec::with_capacity(state.friends.len());
    for link in state.friends {
        if let Some(profile) = load_profile(ctx, link.uid, link.has_gift, link.gift_sent).await? {
            friends.push(profile);
        }
    }
    let mut pending_approvals = Vec::with_capacity(state.pending_approvals.len());
    for uid in state.pending_approvals {
        if let Some(profile) = load_profile(ctx, uid, false, false).await? {
            pending_approvals.push(profile);
        }
    }
    Ok((
        DcNetDataFriendsInfo {
            friends,
            pending_approves: pending_approvals,
        },
        state.claimed_gifts,
    ))
}

async fn social_state(
    ctx: &HandlerContext,
    uid: i64,
) -> NetworkResult<database::models::social::SocialState> {
    Ok(database::db::social::load_state(
        &ctx.state.db,
        uid,
        common::time::day_index(common::time::ServerTime::now_seconds_i32()),
    )
    .await?)
}

async fn load_profile(
    ctx: &HandlerContext,
    uid: i64,
    has_gift: bool,
    gift_sent: bool,
) -> NetworkResult<Option<DcNetDataSocialUserProfile>> {
    Ok(load_player(ctx, uid).await?.map(|player| {
        player.social_profile(
            &ctx.state.tables,
            has_gift,
            gift_sent,
            common::time::ServerTime::now_seconds_i32(),
        )
    }))
}

async fn load_friend_profile(
    ctx: &HandlerContext,
    uid: i64,
    friend_uid: i64,
) -> NetworkResult<Option<DcNetDataSocialUserProfile>> {
    let link = social_state(ctx, uid)
        .await?
        .friends
        .into_iter()
        .find(|link| link.uid == friend_uid);
    match link {
        Some(link) => load_profile(ctx, friend_uid, link.has_gift, link.gift_sent).await,
        None => Ok(None),
    }
}

async fn load_player(ctx: &HandlerContext, uid: i64) -> NetworkResult<Option<Player>> {
    Ok(database::db::player_state::load(&ctx.state.db, uid)
        .await?
        .map(|record| {
            Player::from_record(
                record,
                &ctx.state.tables,
                &common::config().account_defaults,
            )
        }))
}

fn invalid(reason: impl Into<String>) -> NetworkError {
    NetworkError::InvalidSocial(reason.into())
}

#[cfg(test)]
mod tests;
