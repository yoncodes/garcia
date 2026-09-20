use protocol::pbcommon::{
    DcNetDataNpcEffect, DcNetDataRouge, DcNetDataRougeEffect, DcNetDataRougeEvent,
    DcNetDataRougeInfo, DcNetDataRougeNext, DcNetDataRougeRole, DcNetDataRougeRow,
    DcNetDataRougeSubEvt, RougeEffect, RougeEvent, RougeEventStatus,
};

use super::*;

const ROUGE_TECH_CURRENCY_ID: i32 = 100_100_020;

#[derive(Clone, Copy)]
enum RougeStore {
    TradeEvent,
    Subevent,
}

impl Player {
    pub fn available_rouges(&self, tables: &GameTables) -> Vec<DcNetDataRougeRow> {
        let mut rows: Vec<_> = tables
            .rogue_modes
            .rows
            .iter()
            .filter(|config| {
                self.level >= config.required_level
                    && config.unlock_conditions.iter().all(|limit| {
                        limit.key == 46
                            && parse_pass_limit(&limit.value).is_some_and(|(id, count)| {
                                self.rouge_progress.get(&id).copied().unwrap_or(0) >= count
                            })
                    })
            })
            .map(|config| DcNetDataRougeRow {
                rouge_id: config.id,
                pass: self.rouge_progress.get(&config.id).copied().unwrap_or(0),
            })
            .collect();
        rows.sort_by_key(|row| {
            tables
                .rogue_modes
                .get(row.rouge_id)
                .map_or((i32::MAX, i32::MAX), |config| {
                    (config.group_id, config.order)
                })
        });
        rows
    }

    pub fn unlock_rouge_technology(
        &mut self,
        id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataItem, RougeTechnologyError> {
        let config = tables
            .rogue_technology
            .get(id)
            .ok_or(RougeTechnologyError::Unknown(id))?;
        if self.rouge_technology.contains(&id) {
            return Err(RougeTechnologyError::AlreadyUnlocked(id));
        }
        if !config.prerequisites.is_empty()
            && !config
                .prerequisites
                .iter()
                .any(|pre_id| self.rouge_technology.contains(pre_id))
        {
            return Err(RougeTechnologyError::Locked(id));
        }
        if config.cost <= 0 {
            return Err(RougeTechnologyError::InvalidCost(id));
        }

        let costs = BTreeMap::from([(ROUGE_TECH_CURRENCY_ID, config.cost)]);
        let remain = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            RougeTechnologyError::InsufficientCost {
                item_id,
                needed,
                available,
            }
        })?
        .into_iter()
        .next()
        .expect("one Rogue technology cost");
        self.rouge_technology.push(id);
        self.rouge_technology.sort_unstable();
        Ok(remain)
    }

    pub fn claim_rouge_score_rewards(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataTakeRewardRes, i32), RougeScoreRewardError> {
        let rewards: Vec<_> = tables
            .rogue_weekly_rewards
            .iter()
            .filter(|reward| {
                reward.id > self.rouge_score_point_at && reward.required_points <= self.rouge_score
            })
            .map(|reward| (reward.id, reward.reward.key, reward.reward.value))
            .collect();
        let point_at = rewards
            .iter()
            .map(|(id, _, _)| *id)
            .max()
            .ok_or(RougeScoreRewardError::NothingToClaim)?;
        let mut result = DcNetDataTakeRewardRes::default();
        for (_, item_id, amount) in rewards {
            self.add_reward(item_id, amount, tables, now, &mut result);
        }
        self.rouge_score_point_at = point_at;
        Ok((result, point_at))
    }

    pub fn start_rouge(
        &mut self,
        rouge_id: i32,
        roles: Vec<DcNetDataRougeRole>,
        team_equips: Vec<i64>,
        buff_tag: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        if self.rouge_run.is_some() {
            return Err(RougeRunError::AlreadyActive);
        }
        if !self
            .available_rouges(tables)
            .iter()
            .any(|row| row.rouge_id == rouge_id)
        {
            return Err(RougeRunError::Locked(rouge_id));
        }
        self.validate_rouge_roles(&roles)?;
        if let Some(id) = team_equips.iter().find(|id| {
            !self
                .team_equips
                .iter()
                .any(|equipment| equipment.id == **id)
        }) {
            return Err(RougeRunError::UnknownTeamEquip(*id));
        }
        if !tables
            .rogue_rules
            .buff_tag_formula
            .iter()
            .any(|entry| entry.key == buff_tag)
        {
            return Err(RougeRunError::InvalidBuffTag(buff_tag));
        }

        let config = tables
            .rogue_modes
            .get(rouge_id)
            .ok_or(RougeRunError::InvalidConfig(rouge_id))?;
        let first_node = rouge_next(self.uid, config, 0, 0, tables)
            .ok_or(RougeRunError::InvalidConfig(rouge_id))?;
        let coin = tables
            .rogue_rules
            .first_node_coin
            .saturating_add(self.rouge_initial_coin(tables));
        let init_roles = roles
            .iter()
            .map(|role| (role.game_role_id, role.hp))
            .collect();
        let event = DcNetDataRougeEvent {
            event_id: config.initial_event_id,
            buffs: rouge_buff_options(self.uid, buff_tag, 0, &[], tables),
            buff_times: 1,
            status: RougeEventStatus::Res as i32,
            ..Default::default()
        };
        let run = DcNetDataRouge {
            info: Some(DcNetDataRougeInfo {
                rouge_id,
                coin,
                roles,
                first_buff_tag: buff_tag,
                init_roles,
                team_equips,
                ..Default::default()
            }),
            evt: Some(event),
            subevt: Some(DcNetDataRougeSubEvt::default()),
            next: Some(first_node),
        };
        self.rouge_run = Some(run.clone());
        Ok(run)
    }

    pub fn save_rouge_roles(
        &mut self,
        roles: Vec<DcNetDataRougeRole>,
    ) -> Result<Vec<DcNetDataRougeRole>, RougeRunError> {
        self.validate_rouge_roles(&roles)?;
        let info = self
            .rouge_run
            .as_mut()
            .and_then(|run| run.info.as_mut())
            .ok_or(RougeRunError::NotActive)?;
        info.roles = roles.clone();
        Ok(roles)
    }

    pub fn set_rouge_status(&mut self, status: i32) -> Result<i32, RougeRunError> {
        if !matches!(status, 0 | 1) {
            return Err(RougeRunError::InvalidStatus(status));
        }
        let info = self
            .rouge_run
            .as_mut()
            .and_then(|run| run.info.as_mut())
            .ok_or(RougeRunError::NotActive)?;
        info.status = status;
        Ok(status)
    }

    pub fn select_rouge_buff(&mut self, buff_id: i32) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_mut().ok_or(RougeRunError::NotActive)?;
        let event = run.evt.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReBuff as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        if !event.buffs.contains(&buff_id) {
            return Err(RougeRunError::InvalidBuff(buff_id));
        }
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if !info.buffs.contains(&buff_id) {
            info.buffs.push(buff_id);
        }
        event.buffs.retain(|id| *id != buff_id);
        event.buff_times = event.buff_times.saturating_sub(1);
        if event.buff_times == 0 {
            event.status = RougeEventStatus::ResFinished as i32;
            event.buffs.clear();
        }
        Ok(run.clone())
    }

    pub fn waive_rouge_buff(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_mut().ok_or(RougeRunError::NotActive)?;
        let event = run.evt.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReBuff as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        info.coin = info
            .coin
            .saturating_add(tables.rogue_rules.buff_giveup_coin);
        event.buffs.clear();
        event.buff_times = 0;
        event.status = RougeEventStatus::ResFinished as i32;
        Ok(run.clone())
    }

    pub fn regen_rouge_buff(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let event = run.evt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReBuff as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let cost = tables.rogue_rules.buff_reroll_coin;
        if info.coin < cost {
            return Err(RougeRunError::InsufficientRougeCoin {
                needed: cost,
                available: info.coin,
            });
        }
        let regen_times = event.buff_regen_times.saturating_add(1);
        let options = rouge_buff_options(
            self.uid,
            info.first_buff_tag,
            regen_times as u64,
            &info.buffs,
            tables,
        );

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let info = run.info.as_mut().expect("Rogue info was checked");
        info.coin -= cost;
        let event = run.evt.as_mut().expect("Rogue event was checked");
        event.buff_regen_times = regen_times;
        event.buffs = options;
        Ok(run.clone())
    }

    pub fn open_rouge_coinbox(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_mut().ok_or(RougeRunError::NotActive)?;
        let event = run.evt.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReCoinbox as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        let reward = if event.coin > 0 {
            event.coin
        } else {
            tables.rogue_rules.coinbox_default
        };
        info.coin = info.coin.saturating_add(reward);
        event.coin = 0;
        event.status = RougeEventStatus::ResFinished as i32;
        Ok(run.clone())
    }

    pub fn rest_during_rogue_run(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let stage = self
            .rouge_run
            .as_ref()
            .and_then(|run| run.info.as_ref())
            .map(|info| info.rouge_id)
            .ok_or(RougeRunError::NotActive)?;
        let mut pressure_delta = 0;
        let mut hp_rate = 0;
        for effect in &tables.rogue_rules.rest_effect {
            let value = effect
                .value
                .parse::<i32>()
                .map_err(|_| RougeRunError::InvalidConfig(stage))?;
            match RougeEffect::try_from(effect.key).ok() {
                Some(RougeEffect::EffPressureAdd) => {
                    pressure_delta = i32::saturating_add(pressure_delta, value);
                }
                Some(RougeEffect::EffHpAddRate) => {
                    hp_rate = i32::saturating_add(hp_rate, value);
                }
                _ => {}
            }
        }

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let event = run.evt.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReRest as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        info.pressure = info
            .pressure
            .saturating_add(pressure_delta)
            .clamp(0, tables.rogue_rules.pressure_limit.max(0));
        for role in &mut info.roles {
            let max_hp = info
                .init_roles
                .get(&role.game_role_id)
                .copied()
                .unwrap_or(role.hp)
                .max(0);
            let recovery = i64::from(max_hp) * i64::from(hp_rate) / 10_000;
            role.hp = (i64::from(role.hp) + recovery).clamp(0, i64::from(max_hp)) as i32;
        }
        event.status = RougeEventStatus::ResFinished as i32;
        Ok(run.clone())
    }

    pub fn open_rouge_blessing(
        &mut self,
        tables: &GameTables,
    ) -> Result<(DcNetDataRouge, i32), RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let event = run.evt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReBlessing as i32
            || event.status == RougeEventStatus::ResFinished as i32
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let blessing = tables
            .event_blessings
            .get(event.bless_id)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?
            .clone();
        let curse_index = if blessing.curse_effect.is_empty() {
            -1
        } else {
            (mix(self.uid as u64 ^ (info.node_id as u64).rotate_left(13) ^ event.bless_id as u64)
                as usize
                % blessing.curse_effect.len()) as i32
        };
        let stage = info.rouge_id;

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let info = run.info.as_mut().expect("Rogue info was checked");
        if !info.bless_list.contains(&blessing.id) {
            info.bless_list.push(blessing.id);
        }
        for effect in &blessing.blessing_effect {
            let applied =
                apply_immediate_rouge_effect(info, effect, tables.rogue_rules.pressure_limit)
                    .map_err(|()| RougeRunError::InvalidConfig(stage))?;
            if !applied {
                push_rouge_effect(info, effect);
            }
        }
        if let Some(effect) = usize::try_from(curse_index)
            .ok()
            .and_then(|index| blessing.curse_effect.get(index))
        {
            let applied =
                apply_immediate_rouge_effect(info, effect, tables.rogue_rules.pressure_limit)
                    .map_err(|()| RougeRunError::InvalidConfig(stage))?;
            if !applied {
                push_rouge_effect(info, effect);
            }
        }
        run.evt.as_mut().expect("Rogue event was checked").status =
            RougeEventStatus::ResFinished as i32;
        Ok((run.clone(), curse_index))
    }

    pub fn buy_rouge_trade(
        &mut self,
        goods_type: i32,
        goods: &[i32],
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        self.buy_from_rouge_store(RougeStore::TradeEvent, goods_type, goods, tables)
    }

    pub fn buy_rouge_subevent_store(
        &mut self,
        goods_type: i32,
        goods: &[i32],
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        self.buy_from_rouge_store(RougeStore::Subevent, goods_type, goods, tables)
    }

    fn buy_from_rouge_store(
        &mut self,
        store: RougeStore,
        goods_type: i32,
        goods: &[i32],
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        if goods.is_empty() {
            return Err(RougeRunError::InvalidEvent);
        }
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let (regular_goods, random_goods) = match store {
            RougeStore::TradeEvent => {
                let event = run.evt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
                if event.event_id != RougeEvent::ReTrade as i32 {
                    return Err(RougeRunError::InvalidEvent);
                }
                (&event.regular_goods, &event.rand_goods)
            }
            RougeStore::Subevent => {
                let event = run.subevt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
                if event.store_finished {
                    return Err(RougeRunError::InvalidEvent);
                }
                (&event.regular_goods, &event.rand_goods)
            }
        };
        let stock = match goods_type {
            1 => regular_goods,
            2 => random_goods,
            other => return Err(RougeRunError::InvalidRougeGoodsType(other)),
        };
        let mut requested = BTreeMap::<i32, i32>::new();
        for item_id in goods {
            requested
                .entry(*item_id)
                .and_modify(|count| *count = count.saturating_add(1))
                .or_insert(1);
        }
        for (item_id, count) in &requested {
            if stock.get(item_id).copied().unwrap_or_default() < *count {
                return Err(RougeRunError::InsufficientRougeStock(*item_id));
            }
        }
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let base_cost = goods.iter().try_fold(0i64, |total, item_id| {
            let price = tables
                .rogue_items
                .get(*item_id)
                .ok_or(RougeRunError::UnknownRougeItem(*item_id))?
                .price;
            Ok::<_, RougeRunError>(total.saturating_add(i64::from(price.max(0))))
        })?;
        let discount = rouge_effect_total(info, RougeEffect::EffTradeDiscount);
        let cost = (base_cost * i64::from(10_000i32.saturating_add(discount).max(0)) / 10_000)
            .min(i64::from(i32::MAX)) as i32;
        if info.coin < cost {
            return Err(RougeRunError::InsufficientRougeCoin {
                needed: cost,
                available: info.coin,
            });
        }

        let previous_run = self.rouge_run.clone();
        let uid = self.uid;
        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let (regular_goods, random_goods) = match store {
            RougeStore::TradeEvent => {
                let event = run.evt.as_mut().expect("Rogue event was checked");
                (&mut event.regular_goods, &mut event.rand_goods)
            }
            RougeStore::Subevent => {
                let event = run.subevt.as_mut().expect("Rogue subevent was checked");
                (&mut event.regular_goods, &mut event.rand_goods)
            }
        };
        let stock = if goods_type == 1 {
            regular_goods
        } else {
            random_goods
        };
        for (item_id, count) in requested {
            let remain = stock.get(&item_id).copied().unwrap_or_default() - count;
            if remain == 0 {
                stock.remove(&item_id);
            } else {
                stock.insert(item_id, remain);
            }
        }
        let info = run.info.as_mut().expect("Rogue info was checked");
        info.coin -= cost;
        info.total_coin_spend = info.total_coin_spend.saturating_add(cost);
        for (index, item_id) in goods.iter().enumerate() {
            if let Err(error) = acquire_rouge_item(info, uid, *item_id, index as u64, tables) {
                self.rouge_run = previous_run;
                return Err(error);
            }
        }
        Ok(run.clone())
    }

    pub fn use_rouge_item(
        &mut self,
        item_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let item = tables
            .rogue_items
            .get(item_id)
            .ok_or(RougeRunError::UnknownRougeItem(item_id))?
            .clone();
        let uid = self.uid;
        let run = self.rouge_run.as_mut().ok_or(RougeRunError::NotActive)?;
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        let index = info
            .items
            .iter()
            .position(|owned| *owned == item_id)
            .ok_or(RougeRunError::UnknownRougeItem(item_id))?;
        info.items.remove(index);
        apply_rouge_item_effects(info, uid, &item, 0, tables)?;
        Ok(run.clone())
    }

    pub fn claim_rogue_subevent_item(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let uid = self.uid;
        let run = self.rouge_run.as_mut().ok_or(RougeRunError::NotActive)?;
        let subevent = run.subevt.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        if subevent.subevt_item_id == 0 || subevent.item_finished {
            return Err(RougeRunError::InvalidEvent);
        }
        let item_id = subevent.subevt_item_id;
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        let salt = info.total_port_pass as u64;
        acquire_rouge_item(info, uid, item_id, salt, tables)?;
        subevent.item_finished = true;
        Ok(run.clone())
    }

    pub fn select_rouge_npc_effect(
        &mut self,
        npc_effect_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let event = run.evt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReNpc as i32
            || !event
                .npc_effects
                .iter()
                .any(|effect| effect.eff_id == npc_effect_id && effect.enabled)
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let effects = tables
            .event_npc_choice_effects
            .get(npc_effect_id)
            .ok_or(RougeRunError::InvalidConfig(npc_effect_id))?
            .effect_group
            .clone();
        let uid = self.uid;

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let info = run.info.as_mut().ok_or(RougeRunError::InvalidEvent)?;
        let mut battle = 0;
        for (index, effect) in effects.iter().enumerate() {
            match RougeEffect::try_from(effect.key).ok() {
                Some(RougeEffect::EffEnterBattle) => {
                    battle = effect
                        .value
                        .parse::<i32>()
                        .map_err(|_| RougeRunError::InvalidConfig(npc_effect_id))?;
                    info.game_play_id = battle;
                }
                Some(RougeEffect::EffBuffTagRand) => {
                    let (count, quality) = parse_rouge_effect_pair(&effect.value)
                        .ok_or(RougeRunError::InvalidConfig(npc_effect_id))?;
                    add_random_rouge_buffs(
                        info,
                        uid,
                        count,
                        Some(quality),
                        npc_effect_id as u64 ^ index as u64,
                        tables,
                    );
                }
                Some(RougeEffect::EffBuffRand) => {
                    let (count, _) = parse_rouge_effect_pair(&effect.value)
                        .ok_or(RougeRunError::InvalidConfig(npc_effect_id))?;
                    add_random_rouge_buffs(
                        info,
                        uid,
                        count,
                        None,
                        npc_effect_id as u64 ^ index as u64,
                        tables,
                    );
                }
                _ => {
                    let applied = apply_immediate_rouge_effect(
                        info,
                        effect,
                        tables.rogue_rules.pressure_limit,
                    )
                    .map_err(|()| RougeRunError::InvalidConfig(npc_effect_id))?;
                    if !applied {
                        push_rouge_effect(info, effect);
                    }
                }
            }
        }
        let event = run.evt.as_mut().expect("Rogue event was checked");
        if battle == 0 {
            event.status = RougeEventStatus::ResFinished as i32;
        } else {
            event.event_id = RougeEvent::ReBuff as i32;
            event.status = RougeEventStatus::Res as i32;
        }
        Ok(run.clone())
    }

    pub fn enter_rouge(
        &mut self,
        event_pos: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        if run
            .evt
            .as_ref()
            .is_none_or(|event| event.status != RougeEventStatus::ResFinished as i32)
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let config = tables
            .rogue_modes
            .get(info.rouge_id)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?;
        let next = run.next.clone().ok_or(RougeRunError::InvalidEvent)?;
        let index = usize::try_from(event_pos.saturating_sub(1))
            .ok()
            .filter(|index| *index < next.events.len())
            .ok_or(RougeRunError::InvalidEventPosition(event_pos))?;
        let event_id = next.events[index];
        if event_id == RougeEvent::Re as i32 {
            return Err(RougeRunError::InvalidEventPosition(event_pos));
        }
        let pressure = if event_id == RougeEvent::ReBuff as i32 {
            0
        } else {
            let base = tables
                .event_definitions
                .get(event_id)
                .ok_or(RougeRunError::InvalidConfig(event_id))?
                .pressure_increase;
            scale_rouge_amount(
                base,
                rouge_effect_total(info, RougeEffect::EffAllPressureGain),
            )
        };
        let game_play_id = next.game_play_ids.get(index).copied().unwrap_or_default();
        let bless_id = if event_id == RougeEvent::ReBlessing as i32 {
            rouge_blessing(self.uid, next.node_id, &info.bless_list, tables).unwrap_or_default()
        } else {
            0
        };
        let trade_id = if event_id == RougeEvent::ReTrade as i32 {
            rouge_trade(self.uid, next.node_id, tables).unwrap_or_default()
        } else {
            0
        };
        let regular_goods = tables
            .event_trades
            .get(trade_id)
            .map(|trade| {
                trade
                    .item_regular
                    .iter()
                    .map(|item| (item.key, item.value))
                    .collect()
            })
            .unwrap_or_default();
        let npc_id = if event_id == RougeEvent::ReNpc as i32 {
            rouge_npc(self.uid, next.node_id, tables).unwrap_or_default()
        } else {
            0
        };
        let npc_effects = tables
            .event_npcs
            .get(npc_id)
            .map(|npc| {
                npc.choose_effect
                    .iter()
                    .map(|effect_id| DcNetDataNpcEffect {
                        eff_id: *effect_id,
                        enabled: is_rouge_npc_effect_enabled(*effect_id, info, tables),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let following = following_rouge_next(self.uid, config, &next, tables);

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let info = run.info.as_mut().expect("Rogue info was checked");
        info.node_group_id = next.node_group_id;
        info.node_id = next.node_id;
        info.game_play_id = game_play_id;
        info.event_pos = event_pos;
        info.pressure = info
            .pressure
            .saturating_add(pressure)
            .clamp(0, tables.rogue_rules.pressure_limit.max(0));
        run.evt = Some(DcNetDataRougeEvent {
            event_id,
            coin: if event_id == RougeEvent::ReCoinbox as i32 {
                tables.rogue_rules.coinbox_default
            } else {
                0
            },
            bless_id,
            trade_id,
            regular_goods,
            npc_id,
            npc_effects,
            status: if event_id == RougeEvent::ReTrade as i32 {
                RougeEventStatus::ResFinished as i32
            } else {
                RougeEventStatus::Res as i32
            },
            ..Default::default()
        });
        run.subevt = Some(DcNetDataRougeSubEvt::default());
        run.next = following;
        Ok(run.clone())
    }

    pub fn report_rouge_node(
        &mut self,
        roles: Vec<DcNetDataRougeRole>,
        tables: &GameTables,
    ) -> Result<DcNetDataRouge, RougeRunError> {
        self.validate_rouge_roles(&roles)?;
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let event = run.evt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        if event.event_id != RougeEvent::ReBuff as i32 || info.game_play_id == 0 {
            return Err(RougeRunError::InvalidEvent);
        }
        let gameplay = tables
            .rogue_ports
            .get(info.game_play_id)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?;
        let node = tables
            .rogue_nodes
            .get(info.node_id)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?;
        let coin_modifier = rouge_effect_total(info, RougeEffect::EffCoinGain)
            .saturating_add(rouge_effect_total(info, RougeEffect::EffBattleCoinGain));
        let coin = scale_rouge_amount(
            ((gameplay.battle_coin as f32) * node.coin_multiplier) as i32,
            coin_modifier,
        );
        let pressure_base = tables
            .event_definitions
            .get(RougeEvent::ReBuff as i32)
            .ok_or(RougeRunError::InvalidConfig(RougeEvent::ReBuff as i32))?
            .pressure_increase
            .saturating_add(gameplay.gameplay_pressure);
        let pressure_modifier = rouge_effect_total(info, RougeEffect::EffAllPressureGain)
            .saturating_add(rouge_effect_total(info, RougeEffect::EffBattlePressureGain));
        let pressure = scale_rouge_amount(
            ((pressure_base as f32) * node.pressure_multiplier) as i32,
            pressure_modifier,
        );
        let hp_recovery = rouge_effect_total(info, RougeEffect::EffBattleHpRecover);
        let buff_times = gameplay
            .buff_count
            .saturating_add(rouge_effect_total(info, RougeEffect::EffRougeBuffCount))
            .max(1);
        let owned_buffs = info.buffs.clone();
        let tag = info.first_buff_tag;
        let pass = info.total_port_pass.saturating_add(1);
        let options = rouge_buff_options(self.uid, tag, pass as u64, &owned_buffs, tables);
        let subevent_item_id =
            weighted_gameplay(self.uid, info.node_id, pass as usize, &gameplay.battle_item)
                .unwrap_or_default();

        let run = self.rouge_run.as_mut().expect("Rogue run was checked");
        let info = run.info.as_mut().expect("Rogue info was checked");
        info.roles = roles;
        if hp_recovery != 0 {
            apply_rouge_hp(info, |max_hp| {
                i64::from(max_hp) * i64::from(hp_recovery) / 10_000
            });
        }
        info.coin = info.coin.saturating_add(coin);
        info.pressure = info
            .pressure
            .saturating_add(pressure)
            .clamp(0, tables.rogue_rules.pressure_limit.max(0));
        info.total_port_pass = pass;
        consume_rouge_battle_effects(info);
        if gameplay.boss_box_reward != 0 {
            info.total_boss_pass = info.total_boss_pass.saturating_add(1);
        }
        let event = run.evt.as_mut().expect("Rogue event was checked");
        event.status = RougeEventStatus::ResBattleFinished as i32;
        event.buff_times = buff_times;
        event.buffs = options;
        run.subevt = Some(DcNetDataRougeSubEvt {
            subevt_item_id: subevent_item_id,
            ..Default::default()
        });
        Ok(run.clone())
    }

    pub fn end_rouge_run_unsuccessfully(
        &mut self,
        roles: Vec<DcNetDataRougeRole>,
    ) -> Result<DcNetDataRougeInfo, RougeRunError> {
        if !roles.is_empty() {
            self.validate_rouge_roles(&roles)?;
        }
        let mut info = self
            .rouge_run
            .as_ref()
            .ok_or(RougeRunError::NotActive)?
            .info
            .clone()
            .ok_or(RougeRunError::InvalidEvent)?;
        if !roles.is_empty() {
            info.roles = roles;
        }
        self.rouge_run = None;
        Ok(info)
    }

    pub fn open_rouge_boss_box(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataTakeRewardRes, DcNetDataItem), RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let info = run.info.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        let subevent = run.subevt.as_ref().ok_or(RougeRunError::InvalidEvent)?;
        if subevent.boss_box_opened {
            return Err(RougeRunError::BossBoxAlreadyOpened);
        }
        if info.game_play_id == 0 {
            return Err(RougeRunError::InvalidEvent);
        }
        let gameplay = tables
            .rogue_ports
            .get(info.game_play_id)
            .ok_or(RougeRunError::InvalidConfig(info.game_play_id))?;
        if gameplay.boss_box_reward == 0 {
            return Err(RougeRunError::InvalidEvent);
        }
        // This id behaves as a server-side reward pool. Do not charge the
        // player unless the extracted data can resolve it deterministically.
        let reward_entries = tables
            .rewards
            .get(gameplay.boss_box_reward)
            .ok_or(RougeRunError::InvalidConfig(gameplay.boss_box_reward))?
            .reward
            .clone();
        let item_cost = &tables.rogue_rules.bossbox_item_cost;
        let available_item = self
            .items
            .iter()
            .find(|item| item.item_id == item_cost.key)
            .map_or(0, |item| item.amount);
        let cost = if available_item >= item_cost.value {
            (item_cost.key, item_cost.value)
        } else {
            let energy_item_id = tables.cultivation_constants.heat_item_id;
            let available_energy = self
                .items
                .iter()
                .find(|item| item.item_id == energy_item_id)
                .map_or(0, |item| item.amount);
            if available_energy < tables.rogue_rules.bossbox_phy_cost {
                return Err(RougeRunError::InsufficientBossBoxCost);
            }
            (energy_item_id, tables.rogue_rules.bossbox_phy_cost)
        };
        let remain = consume_item_costs(&mut self.items, &BTreeMap::from([cost]), |_, _, _| {
            RougeRunError::InsufficientBossBoxCost
        })?
        .into_iter()
        .next()
        .expect("one Rogue boss-box cost");

        let mut reward = DcNetDataTakeRewardRes::default();
        for entry in reward_entries {
            self.add_reward(entry.key, entry.value, tables, now, &mut reward);
        }
        self.rouge_run
            .as_mut()
            .and_then(|run| run.subevt.as_mut())
            .expect("Rogue subevent was checked")
            .boss_box_opened = true;
        Ok((reward, remain))
    }

    pub fn finish_rouge(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<RougeFinishOutcome, RougeRunError> {
        let run = self.rouge_run.as_ref().ok_or(RougeRunError::NotActive)?;
        let info = run.info.clone().ok_or(RougeRunError::InvalidEvent)?;
        if run.next.is_some()
            || run
                .evt
                .as_ref()
                .is_none_or(|event| event.status != RougeEventStatus::ResFinished as i32)
        {
            return Err(RougeRunError::InvalidEvent);
        }
        let config = tables
            .rogue_modes
            .get(info.rouge_id)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?;
        let first_clear = self
            .rouge_progress
            .get(&info.rouge_id)
            .copied()
            .unwrap_or_default()
            == 0;
        let exchanged_coin = (i64::from(info.coin.max(0))
            * i64::from(config.coin_exchange_rate.max(0))
            / i64::from(tables.rogue_rules.coin_ex_basic_rate.max(1)))
        .min(i64::from(i32::MAX)) as i32;
        let technology_points = config.tech_reward.max(0).saturating_add(exchanged_coin);

        let mut reward = DcNetDataTakeRewardRes::default();
        if first_clear {
            for drop in &config.first_clear_rewards {
                self.add_reward(drop.key, drop.value, tables, now, &mut reward);
            }
        }
        let technology = self
            .add_item(ROUGE_TECH_CURRENCY_ID, technology_points, tables)
            .ok_or(RougeRunError::InvalidConfig(info.rouge_id))?;
        self.rouge_score = self.rouge_score.saturating_add(config.point_reward.max(0));
        self.rouge_progress
            .entry(info.rouge_id)
            .and_modify(|pass| *pass = pass.saturating_add(1))
            .or_insert(1);
        self.rouge_run = None;

        Ok(RougeFinishOutcome {
            rouge_score: self.rouge_score,
            reward,
            items: vec![technology],
            rouge_info: info,
        })
    }

    fn validate_rouge_roles(&self, roles: &[DcNetDataRougeRole]) -> Result<(), RougeRunError> {
        if !(1..=3).contains(&roles.len()) {
            return Err(RougeRunError::InvalidRoleCount);
        }
        for (index, role) in roles.iter().enumerate() {
            if !self.owns_role(role.game_role_id) {
                return Err(RougeRunError::UnknownRole(role.game_role_id));
            }
            if roles[..index]
                .iter()
                .any(|current| current.game_role_id == role.game_role_id)
            {
                return Err(RougeRunError::DuplicateRole(role.game_role_id));
            }
        }
        Ok(())
    }

    fn rouge_initial_coin(&self, tables: &GameTables) -> i32 {
        self.rouge_technology
            .iter()
            .filter_map(|id| tables.rogue_technology.get(*id))
            .flat_map(|technology| &technology.effects)
            .filter(|effect| effect.key == 17)
            .filter_map(|effect| effect.value.split('|').next()?.parse::<i32>().ok())
            .fold(0, i32::saturating_add)
    }
}

fn rouge_next(
    uid: i64,
    config: &configs::tables::RogueMode,
    group_index: usize,
    node_index: usize,
    tables: &GameTables,
) -> Option<DcNetDataRougeNext> {
    let group_id = *config.node_groups.get(group_index)?;
    let group = tables.rogue_node_groups.get(group_id)?;
    let node_id = *group.nodes.get(node_index)?;
    let node = tables.rogue_nodes.get(node_id)?;
    let events: Vec<_> = std::iter::once(&node.primary_event)
        .chain(node.secondary_event.iter())
        .chain(node.tertiary_event.iter())
        .map(|event| event.key)
        .collect();
    let game_play_ids = events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            if *event == RougeEvent::ReBuff as i32 {
                weighted_gameplay(uid, node_id, index, &node.gameplay_choices).unwrap_or_default()
            } else {
                0
            }
        })
        .collect();
    Some(DcNetDataRougeNext {
        node_group_id: group_id,
        node_id,
        events,
        game_play_ids,
    })
}

fn following_rouge_next(
    uid: i64,
    config: &configs::tables::RogueMode,
    current: &DcNetDataRougeNext,
    tables: &GameTables,
) -> Option<DcNetDataRougeNext> {
    let group_index = config
        .node_groups
        .iter()
        .position(|group_id| *group_id == current.node_group_id)?;
    let group = tables.rogue_node_groups.get(current.node_group_id)?;
    let node_index = group
        .nodes
        .iter()
        .position(|node_id| *node_id == current.node_id)?;
    rouge_next(uid, config, group_index, node_index + 1, tables)
        .or_else(|| rouge_next(uid, config, group_index + 1, 0, tables))
}

fn weighted_gameplay(
    uid: i64,
    node_id: i32,
    index: usize,
    choices: &[configs::tables::TablePair<i32>],
) -> Option<i32> {
    let total: u64 = choices
        .iter()
        .map(|choice| choice.value.max(0) as u64)
        .sum();
    if total == 0 {
        return None;
    }
    let mut pick = mix(uid as u64 ^ (node_id as u64).rotate_left(19) ^ index as u64) % total;
    for choice in choices {
        let weight = choice.value.max(0) as u64;
        if pick < weight {
            return Some(choice.key);
        }
        pick -= weight;
    }
    None
}

fn rouge_buff_options(
    uid: i64,
    tag: i32,
    salt: u64,
    excluded: &[i32],
    tables: &GameTables,
) -> Vec<i32> {
    let choices: Vec<_> = tables
        .rogue_buffs
        .rows
        .iter()
        .filter(|buff| buff.tag == tag && !excluded.contains(&buff.id))
        .collect();
    if choices.is_empty() {
        return Vec::new();
    }
    let offset = (mix(uid as u64 ^ salt) as usize) % choices.len();
    (0..3.min(choices.len()))
        .map(|index| choices[(offset + index) % choices.len()].id)
        .collect()
}

fn rouge_blessing(uid: i64, node_id: i32, owned: &[i32], tables: &GameTables) -> Option<i32> {
    let choices: Vec<_> = tables
        .event_blessings
        .rows
        .iter()
        .filter(|blessing| !owned.contains(&blessing.id))
        .collect();
    (!choices.is_empty()).then(|| {
        let index = mix(uid as u64 ^ (node_id as u64).rotate_left(7)) as usize % choices.len();
        choices[index].id
    })
}

fn rouge_trade(uid: i64, node_id: i32, tables: &GameTables) -> Option<i32> {
    let choices = &tables.event_trades.rows;
    (!choices.is_empty()).then(|| {
        let index = mix(uid as u64 ^ (node_id as u64).rotate_left(23)) as usize % choices.len();
        choices[index].id
    })
}

fn rouge_npc(uid: i64, node_id: i32, tables: &GameTables) -> Option<i32> {
    let choices = &tables.event_npcs.rows;
    (!choices.is_empty()).then(|| {
        let index = mix(uid as u64 ^ (node_id as u64).rotate_left(29)) as usize % choices.len();
        choices[index].id
    })
}

fn is_rouge_npc_effect_enabled(
    effect_id: i32,
    info: &DcNetDataRougeInfo,
    tables: &GameTables,
) -> bool {
    tables
        .event_npc_choices
        .get(effect_id)
        .is_some_and(|choice| {
            choice
                .choose_condition
                .iter()
                .all(|condition| match condition.key {
                    3 => info.coin >= condition.value,
                    5 => info.roles.iter().all(|role| {
                        let max_hp = info
                            .init_roles
                            .get(&role.game_role_id)
                            .copied()
                            .unwrap_or(role.hp)
                            .max(1);
                        i64::from(role.hp) * 10_000 / i64::from(max_hp)
                            >= i64::from(condition.value)
                    }),
                    _ => false,
                })
        })
}

fn acquire_rouge_item(
    info: &mut DcNetDataRougeInfo,
    uid: i64,
    item_id: i32,
    salt: u64,
    tables: &GameTables,
) -> Result<(), RougeRunError> {
    let item = tables
        .rogue_items
        .get(item_id)
        .ok_or(RougeRunError::UnknownRougeItem(item_id))?;
    if item.auto_use != 0 {
        apply_rouge_item_effects(info, uid, item, salt, tables)
    } else {
        info.items.push(item_id);
        Ok(())
    }
}

fn apply_rouge_item_effects(
    info: &mut DcNetDataRougeInfo,
    uid: i64,
    item: &configs::tables::RogueItem,
    salt: u64,
    tables: &GameTables,
) -> Result<(), RougeRunError> {
    for (index, effect) in item.effects.iter().enumerate() {
        let effect_salt = salt ^ (item.id as u64).rotate_left(17) ^ index as u64;
        match RougeEffect::try_from(effect.key).ok() {
            Some(RougeEffect::EffBuffTagRand) => {
                let (count, quality) = parse_rouge_effect_pair(&effect.value)
                    .ok_or(RougeRunError::InvalidConfig(item.id))?;
                add_random_rouge_buffs(info, uid, count, Some(quality), effect_salt, tables);
            }
            Some(RougeEffect::EffBuffRand) => {
                let (count, _) = parse_rouge_effect_pair(&effect.value)
                    .ok_or(RougeRunError::InvalidConfig(item.id))?;
                add_random_rouge_buffs(info, uid, count, None, effect_salt, tables);
            }
            _ => {
                let applied =
                    apply_immediate_rouge_effect(info, effect, tables.rogue_rules.pressure_limit)
                        .map_err(|()| RougeRunError::InvalidConfig(item.id))?;
                if !applied {
                    push_rouge_item_effect(info, effect);
                }
            }
        }
    }
    Ok(())
}

fn add_random_rouge_buffs(
    info: &mut DcNetDataRougeInfo,
    uid: i64,
    count: i32,
    quality: Option<i32>,
    salt: u64,
    tables: &GameTables,
) {
    let choices: Vec<_> = tables
        .rogue_buffs
        .rows
        .iter()
        .filter(|buff| quality.is_none_or(|quality| buff.quality == quality))
        .filter(|buff| !info.buffs.contains(&buff.id))
        .collect();
    if choices.is_empty() {
        return;
    }
    let offset = mix(uid as u64 ^ salt) as usize % choices.len();
    let count = usize::try_from(count.max(0))
        .unwrap_or_default()
        .min(choices.len());
    for index in 0..count {
        info.buffs
            .push(choices[(offset + index) % choices.len()].id);
    }
}

fn rouge_effect_total(info: &DcNetDataRougeInfo, effect: RougeEffect) -> i32 {
    info.eff_list
        .iter()
        .filter(|current| current.eff_id == effect as i32)
        .filter_map(|current| current.eff_params.split('|').next()?.parse::<i32>().ok())
        .fold(0, i32::saturating_add)
}

fn scale_rouge_amount(base: i32, modifier: i32) -> i32 {
    (i64::from(base) * i64::from(10_000i32.saturating_add(modifier).max(0)) / 10_000)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn consume_rouge_battle_effects(info: &mut DcNetDataRougeInfo) {
    for effect in &mut info.eff_list {
        if !effect.eternal && effect.remain_cnt > 0 {
            effect.remain_cnt -= 1;
        }
    }
    info.eff_list
        .retain(|effect| effect.eternal || effect.remain_cnt > 0);
}

fn parse_rouge_effect_pair(value: &str) -> Option<(i32, i32)> {
    let (first, second) = value.split_once('|')?;
    Some((first.parse().ok()?, second.parse().ok()?))
}

fn apply_immediate_rouge_effect(
    info: &mut DcNetDataRougeInfo,
    effect: &configs::tables::TablePair<String>,
    pressure_limit: i32,
) -> Result<bool, ()> {
    let kind = RougeEffect::try_from(effect.key).ok();
    if kind == Some(RougeEffect::Eff) {
        return Ok(true);
    }
    if !matches!(
        kind,
        Some(
            RougeEffect::EffPressureAdd
                | RougeEffect::EffPressureAddRate
                | RougeEffect::EffHpAddRate
                | RougeEffect::EffHpAddValue
                | RougeEffect::EffCoinAddValue
        )
    ) {
        return Ok(false);
    }
    let value = effect
        .value
        .split('|')
        .next()
        .ok_or(())?
        .parse::<i32>()
        .map_err(|_| ())?;
    match kind {
        Some(RougeEffect::EffPressureAdd) => {
            info.pressure = info
                .pressure
                .saturating_add(value)
                .clamp(0, pressure_limit.max(0));
        }
        Some(RougeEffect::EffPressureAddRate) => {
            info.pressure = (i64::from(info.pressure)
                * i64::from(10_000i32.saturating_add(value).max(0))
                / 10_000)
                .clamp(0, i64::from(pressure_limit.max(0))) as i32;
        }
        Some(RougeEffect::EffHpAddRate) => {
            apply_rouge_hp(info, |max_hp| i64::from(max_hp) * i64::from(value) / 10_000)
        }
        Some(RougeEffect::EffHpAddValue) => apply_rouge_hp(info, |_| i64::from(value)),
        Some(RougeEffect::EffCoinAddValue) => {
            info.coin = info.coin.saturating_add(value).max(0);
        }
        _ => unreachable!("numeric Rogue effects were filtered"),
    }
    Ok(true)
}

fn apply_rouge_hp(info: &mut DcNetDataRougeInfo, delta: impl Fn(i32) -> i64) {
    for role in &mut info.roles {
        let max_hp = info
            .init_roles
            .get(&role.game_role_id)
            .copied()
            .unwrap_or(role.hp)
            .max(0);
        role.hp = (i64::from(role.hp) + delta(max_hp)).clamp(0, i64::from(max_hp)) as i32;
    }
}

fn push_rouge_effect(info: &mut DcNetDataRougeInfo, effect: &configs::tables::TablePair<String>) {
    let id = info
        .eff_list
        .iter()
        .map(|effect| effect.id)
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    info.eff_list.push(DcNetDataRougeEffect {
        id,
        eff_id: effect.key,
        eff_params: effect.value.clone(),
        eternal: true,
        ..Default::default()
    });
}

fn push_rouge_item_effect(
    info: &mut DcNetDataRougeInfo,
    effect: &configs::tables::TablePair<String>,
) {
    let count = effect
        .value
        .split_once('|')
        .and_then(|(_, count)| count.parse::<i32>().ok())
        .unwrap_or_default()
        .max(0);
    let id = info
        .eff_list
        .iter()
        .map(|effect| effect.id)
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    info.eff_list.push(DcNetDataRougeEffect {
        id,
        eff_id: effect.key,
        eff_params: effect.value.clone(),
        remain_cnt: count,
        eternal: count == 0,
        total_cnt: count,
    });
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn parse_pass_limit(value: &str) -> Option<(i32, i32)> {
    let (id, count) = value.split_once('|')?;
    Some((id.parse().ok()?, count.parse().ok()?))
}

#[cfg(test)]
mod tests;
