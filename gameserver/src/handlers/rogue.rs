use protocol::{
    cs::{
        DcNetWorkingParamQuestionUrl, DcNetWorkingParamRougeBlessOpen,
        DcNetWorkingParamRougeBossBoxOpen, DcNetWorkingParamRougeBuffSelect,
        DcNetWorkingParamRougeCoinboxOpen, DcNetWorkingParamRougeEnter,
        DcNetWorkingParamRougeFinish, DcNetWorkingParamRougeFormSave, DcNetWorkingParamRougeInfo,
        DcNetWorkingParamRougeItemUse, DcNetWorkingParamRougeList,
        DcNetWorkingParamRougeNodeReport, DcNetWorkingParamRougeNpcSelectEffect,
        DcNetWorkingParamRougeRegenBuff, DcNetWorkingParamRougeRest,
        DcNetWorkingParamRougeScoreRewardClaim, DcNetWorkingParamRougeStart,
        DcNetWorkingParamRougeStatusSet, DcNetWorkingParamRougeSubevtItem,
        DcNetWorkingParamRougeSubevtStoreBuy, DcNetWorkingParamRougeTechTake,
        DcNetWorkingParamRougeTechTree, DcNetWorkingParamRougeTradeBuy,
        DcNetWorkingParamRougeWaiveBuff, DcNetWorkingResQuestionUrl, DcNetWorkingResRougeBlessOpen,
        DcNetWorkingResRougeBossBoxOpen, DcNetWorkingResRougeBuffSelect,
        DcNetWorkingResRougeCoinboxOpen, DcNetWorkingResRougeEnter, DcNetWorkingResRougeFinish,
        DcNetWorkingResRougeFormSave, DcNetWorkingResRougeInfo, DcNetWorkingResRougeItemUse,
        DcNetWorkingResRougeList, DcNetWorkingResRougeNodeReport,
        DcNetWorkingResRougeNpcSelectEffect, DcNetWorkingResRougeRegenBuff,
        DcNetWorkingResRougeRest, DcNetWorkingResRougeScoreRewardClaim, DcNetWorkingResRougeStart,
        DcNetWorkingResRougeStatusSet, DcNetWorkingResRougeSubevtItem,
        DcNetWorkingResRougeSubevtStoreBuy, DcNetWorkingResRougeTechTake,
        DcNetWorkingResRougeTechTree, DcNetWorkingResRougeTradeBuy, DcNetWorkingResRougeWaiveBuff,
    },
    pbcommon::DcNetDataRouge,
    prost::Message,
};

use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub async fn on_rogue_list(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeList::decode(request.payload.as_slice())?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRougeList {
            list: ctx.player()?.available_rouges(&ctx.state.tables),
        },
    )
}

pub async fn on_info(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeInfo::decode(request.payload.as_slice())?;
    let config = common::config();
    let player = ctx.player()?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRougeInfo {
            rouge: Some(
                player
                    .rouge_run
                    .clone()
                    .unwrap_or_else(DcNetDataRouge::default),
            ),
            talent_list: player.rouge_technology.clone(),
            score: player.rouge_score,
            score_point_at: player.rouge_score_point_at,
            score_cd: common::time::next_daily_refresh_seconds(
                common::time::ServerTime::now_seconds_i32(),
                config.server.zone_offset,
                ctx.state.tables.cultivation_constants.daily_reset_hour,
            ),
        },
    )
}

pub async fn on_start(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeStart::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .start_rouge(
            request.rouge_id,
            request.roles,
            request.team_equips,
            request.buff_tag,
            tables,
        )
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeStart {
            rouge: Some(rouge),
            coin_item: None,
        },
    )
}

pub async fn on_form_save(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeFormSave::decode(packet.payload.as_slice())?;
    let roles = ctx
        .update_player()?
        .save_rouge_roles(request.roles)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeFormSave { roles })
}

pub async fn on_status_set(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeStatusSet::decode(packet.payload.as_slice())?;
    let status = ctx
        .update_player()?
        .set_rouge_status(request.status)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeStatusSet { status })
}

pub async fn on_enter(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeEnter::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .enter_rouge(request.event_pos, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeEnter { rouge: Some(rouge) })
}

pub async fn on_node_report(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeNodeReport::decode(packet.payload.as_slice())?;
    if !request.success {
        let rouge_info = ctx
            .update_player()?
            .end_rouge_run_unsuccessfully(request.roles)
            .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
        let rouge_score = ctx.player()?.rouge_score;
        return ctx.send_reply(
            &packet,
            DcNetWorkingResRougeNodeReport {
                success: false,
                rouge_score,
                items: Vec::new(),
                rouge_info: Some(rouge_info),
                rouge: None,
            },
        );
    }
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .report_rouge_node(request.roles, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    let rouge_score = ctx.player()?.rouge_score;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeNodeReport {
            success: true,
            rouge_score,
            items: Vec::new(),
            rouge_info: None,
            rouge: Some(rouge),
        },
    )
}

pub async fn on_buff_select(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeBuffSelect::decode(packet.payload.as_slice())?;
    let rouge = ctx
        .update_player()?
        .select_rouge_buff(request.buff_id)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeBuffSelect { rouge: Some(rouge) },
    )
}

pub async fn on_buff_waive(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeWaiveBuff::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .waive_rouge_buff(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeWaiveBuff { rouge: Some(rouge) },
    )
}

pub async fn on_buff_regen(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeRegenBuff::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .regen_rouge_buff(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeRegenBuff { rouge: Some(rouge) },
    )
}

pub async fn on_coinbox_open(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeCoinboxOpen::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .open_rouge_coinbox(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeCoinboxOpen { rouge: Some(rouge) },
    )
}

pub async fn on_rest(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeRest::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .rest_during_rogue_run(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeRest { rouge: Some(rouge) })
}

pub async fn on_bless_open(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeBlessOpen::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (rouge, curse_index) = ctx
        .update_player()?
        .open_rouge_blessing(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeBlessOpen {
            rouge: Some(rouge),
            curse_index,
        },
    )
}

pub async fn on_finish(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeFinish::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let outcome = ctx
        .update_player()?
        .finish_rouge(tables, common::time::ServerTime::now_seconds_i32())
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeFinish {
            rouge_score: outcome.rouge_score,
            reward: Some(outcome.reward),
            items: outcome.items,
            rouge_info: Some(outcome.rouge_info),
        },
    )
}

pub async fn on_trade_buy(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeTradeBuy::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .buy_rouge_trade(request.goods_type, &request.goods, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeTradeBuy { rouge: Some(rouge) })
}

pub async fn on_item_use(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeItemUse::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .use_rouge_item(request.item_id, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(&packet, DcNetWorkingResRougeItemUse { rouge: Some(rouge) })
}

pub async fn on_sub_event_item(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamRougeSubevtItem::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .claim_rogue_subevent_item(tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeSubevtItem { rouge: Some(rouge) },
    )
}

pub async fn on_npc_select_effect(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeNpcSelectEffect::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .select_rouge_npc_effect(request.npc_eff_id, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeNpcSelectEffect { rouge: Some(rouge) },
    )
}

pub async fn on_score_reward_claim(
    ctx: &mut HandlerContext,
    packet: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamRougeScoreRewardClaim::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let (reward, score_point_at) = ctx
        .update_player()?
        .claim_rouge_score_rewards(tables, common::time::ServerTime::now_seconds_i32())
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeScoreRewardClaim {
            reward: Some(reward),
            score_point_at,
        },
    )
}

pub async fn on_question_url(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamQuestionUrl::decode(request.payload.as_slice())?;
    ctx.send_reply(
        &request,
        DcNetWorkingResQuestionUrl {
            url: common::config().gameplay.question_url.clone(),
        },
    )
}

pub async fn on_tech_tree(ctx: &mut HandlerContext, request: ClientPacket) -> NetworkResult<()> {
    DcNetWorkingParamRougeTechTree::decode(request.payload.as_slice())?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRougeTechTree {
            list: ctx.player()?.rouge_technology.clone(),
        },
    )
}

pub async fn on_tech_take(ctx: &mut HandlerContext, packet: ClientPacket) -> NetworkResult<()> {
    let request = DcNetWorkingParamRougeTechTake::decode(packet.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let remain = ctx
        .update_player()?
        .unlock_rouge_technology(request.id, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &packet,
        DcNetWorkingResRougeTechTake {
            id: request.id,
            remain: Some(remain),
        },
    )
}

pub async fn on_rouge_boss_box_open(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    DcNetWorkingParamRougeBossBoxOpen::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let (reward, remain) = ctx
        .update_player()?
        .open_rouge_boss_box(&state.tables, common::time::ServerTime::now_seconds_i32())
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRougeBossBoxOpen {
            reward: Some(reward),
            remain: Some(remain),
        },
    )
}

pub async fn on_subevt_store_buy(
    ctx: &mut HandlerContext,
    request: ClientPacket,
) -> NetworkResult<()> {
    let param = DcNetWorkingParamRougeSubevtStoreBuy::decode(request.payload.as_slice())?;
    let state = ctx.state.clone();
    let tables = &state.tables;
    let rouge = ctx
        .update_player()?
        .buy_rouge_subevent_store(param.goods_type, &param.goods, tables)
        .map_err(|error| NetworkError::InvalidRouge(error.to_string()))?;
    ctx.send_reply(
        &request,
        DcNetWorkingResRougeSubevtStoreBuy { rouge: Some(rouge) },
    )
}
