use protocol::{
    cs::{
        DcNetWorkingNotifyProfileNew, DcNetWorkingParamRoleRankUpAward,
        DcNetWorkingParamRolesBrekUp, DcNetWorkingParamRolesChangeSkin,
        DcNetWorkingParamRolesLevelUp, DcNetWorkingParamRolesRankUp,
        DcNetWorkingParamRolesUserRoles, DcNetWorkingParamTalentLevelUp,
        DcNetWorkingResRoleRankUpAward, DcNetWorkingResRolesBrekUp, DcNetWorkingResRolesChangeSkin,
        DcNetWorkingResRolesLevelUp, DcNetWorkingResRolesRankUp, DcNetWorkingResRolesUserRoles,
        DcNetWorkingResTalentLevelUp,
    },
    pbcommon::Reddot,
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_user_roles(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRolesUserRoles::decode(request.payload.as_slice())?;
    let role_details_list = ctx.player()?.roles.clone();
    ctx.send_reply(
        &request,
        DcNetWorkingResRolesUserRoles { role_details_list },
    )
}

pub async fn on_change_skin(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesChangeSkin::decode(packet.payload.as_slice())?;
    let (player, tables) = ctx.update_player()?.with_tables();
    player
        .set_role_skin(request.game_role_id, request.skin_id, tables)
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesChangeSkin {
            game_role_id: request.game_role_id,
            skin_id: request.skin_id,
        },
    )
}

pub async fn on_level_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesLevelUp::decode(packet.payload.as_slice())?;
    let now = common::time::ServerTime::now_seconds_i32();
    let (player, tables) = ctx.update_player()?.with_tables();
    let outcome = player
        .level_up_role(
            request.game_role_id,
            &request.itmes,
            tables,
            now,
            common::config().server.zone_offset,
        )
        .map_err(|error| NetworkError::InvalidRoleMutation(error.to_string()))?;
    super::battle_pass::push_task_updates(ctx, &outcome.battle_pass_updates)?;
    super::achievement::push_updates(ctx, &outcome.achievement_updates)?;
    super::activity::push_seven_day_updates(ctx, &outcome.seven_day_updates)?;
    super::tasks::push_daily_updates(ctx, &outcome.daily_updates)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesLevelUp {
            user_up_data: Some(outcome.user_up_data),
            game_role_id: request.game_role_id,
            remain: outcome.remain,
            skill_points: 0,
            talents: outcome.talents,
        },
    )
}

pub async fn on_rank_up(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesRankUp::decode(packet.payload.as_slice())?;
    let role_id = request
        .data
        .ok_or_else(|| NetworkError::InvalidRoleMutation("missing role id".into()))?
        .game_role_id;
    let (player, tables) = ctx.update_player()?.with_tables();
    let outcome = player.rank_up_role(role_id, tables).map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesRankUp {
            game_role_id: outcome.game_role_id,
            position: outcome.position,
            items: outcome.items,
            talents: outcome.talents,
            skill_unlocked: Vec::new(),
        },
    )
}

pub async fn on_resonance(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRolesBrekUp::decode(packet.payload.as_slice())?;
    let (player, tables) = ctx.update_player()?.with_tables();
    let (maid_quality, remain, namecard) = player
        .resonate_role(request.game_role_id, tables)
        .map_err(invalid_role)?;
    if let Some(card) = namecard {
        super::red_dot::push_one(ctx, Reddot::Card, card.id)?;
        ctx.push(
            protocol::proids::NotifyId::DcNetWorkingNotifyProfileNew as u16,
            DcNetWorkingNotifyProfileNew {
                avatar: None,
                card: Some(card),
            },
        )?;
    }
    ctx.send_reply(
        &packet,
        DcNetWorkingResRolesBrekUp {
            game_role_id: request.game_role_id,
            maid_quality,
            remain,
        },
    )
}

pub async fn on_rank_reward(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRoleRankUpAward::decode(packet.payload.as_slice())?;
    let (player, tables) = ctx.update_player()?.with_tables();
    let reward = player
        .claim_role_rank_reward(
            request.rid,
            request.level,
            tables,
            common::time::ServerTime::now_seconds_i32(),
        )
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRoleRankUpAward {
            reward: Some(reward),
        },
    )
}

pub async fn on_talent_level_up(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamTalentLevelUp::decode(packet.payload.as_slice())?;
    let (player, tables) = ctx.update_player()?.with_tables();
    let (talent, remain) = player
        .level_up_role_talent(request.game_role_id, request.pos, tables)
        .map_err(invalid_role)?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResTalentLevelUp {
            talent: Some(talent),
            remain,
        },
    )
}

fn invalid_role(error: crate::logic::RoleMutationError) -> NetworkError {
    NetworkError::InvalidRoleMutation(error.to_string())
}
