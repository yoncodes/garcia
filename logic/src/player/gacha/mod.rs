use super::*;

impl Player {
    pub fn gacha_pools(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataGachaData> {
        self.gacha_pools_with_overrides(tables, now, zone_offset, &[])
    }

    pub fn gacha_pools_with_overrides(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
        banner_overrides: &[(i32, i32)],
    ) -> Vec<DcNetDataGachaData> {
        let mut limited: Vec<_> = tables
            .gacha_pools
            .rows
            .iter()
            .filter(|config| {
                config.time_type != 0
                    && (draw_seconds_remaining(config, now, zone_offset).is_some()
                        || banner_override_expiration(banner_overrides, config.id, now).is_some())
            })
            .collect();
        limited.sort_by_key(|config| config.id);
        let mut permanent: Vec<_> = tables
            .gacha_pools
            .rows
            .iter()
            .filter(|config| config.time_type == 0)
            .collect();
        permanent.sort_by_key(|config| config.id);
        limited
            .into_iter()
            .chain(permanent)
            .map(|config| {
                self.gacha_info(
                    config,
                    tables,
                    now,
                    zone_offset,
                    true,
                    banner_override_expiration(banner_overrides, config.id, now),
                )
            })
            .collect()
    }

    pub fn draw_gacha(
        &mut self,
        gacha_id: i32,
        count: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<GachaOutcome, GachaError> {
        self.draw_gacha_with_override(gacha_id, count, tables, now, zone_offset, None)
    }

    pub fn draw_gacha_with_override(
        &mut self,
        gacha_id: i32,
        count: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
        override_expires_at: Option<i32>,
    ) -> Result<GachaOutcome, GachaError> {
        if !matches!(count, 1 | 10) {
            return Err(GachaError::InvalidCount(count));
        }
        let config = tables
            .gacha_pools
            .get(gacha_id)
            .cloned()
            .ok_or(GachaError::Unknown(gacha_id))?;
        if config.time_type != 0
            && gacha_seconds_remaining(&config, now, zone_offset, override_expires_at).is_none()
        {
            return Err(GachaError::Inactive(gacha_id));
        }
        let needed = config.single_draw_cost.value.saturating_mul(count);
        let available = self
            .items
            .iter()
            .find(|item| item.item_id == config.single_draw_cost.key)
            .map_or(0, |item| item.amount);
        let ticket_cost = available.min(needed);
        let diamond_cost = needed
            .saturating_sub(ticket_cost)
            .saturating_mul(tables.cultivation_constants.diamonds_per_stamp);
        let mut costs = BTreeMap::new();
        if ticket_cost > 0 {
            costs.insert(config.single_draw_cost.key, ticket_cost);
        }
        if diamond_cost > 0 {
            costs.insert(tables.cultivation_constants.diamond_item_id, diamond_cost);
        }
        let previous_battle_pass = self.battle_pass_task_snapshot(tables, now, zone_offset);
        let remains = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            GachaError::InsufficientCost {
                item_id,
                needed,
                available,
            }
        })?;

        let mut pity_count = self.gacha_pity_count(&config, tables);
        let state_index = self
            .gachas
            .iter()
            .position(|state| state.gacha_id == gacha_id)
            .unwrap_or_else(|| {
                self.gachas.push(GachaState {
                    gacha_id,
                    all_count: 0,
                    gacha_count_10: pity_count,
                    reward_cnt: 0,
                    reward_num: 0,
                    taken_new_reward: false,
                    had_take_reward: false,
                });
                self.gachas.len() - 1
            });
        let old_count = self.gachas[state_index].all_count;
        let mut reward_list = Vec::with_capacity(count as usize);
        let mut coin = None;
        let mut events = Vec::new();
        let mut previous_achievements = self.achievements(tables, now);
        let mut unlocked_archives = self.unlocked_archive_ids(tables, now);
        for offset in 0..count {
            let draw_number = old_count.saturating_add(offset).saturating_add(1);
            let guaranteed = config.guaranteed_draw_interval > 0
                && pity_count.saturating_add(1) >= config.guaranteed_draw_interval;
            let source_reward_id = if guaranteed {
                config.ten_reward_id
            } else {
                config.reward_id
            };
            let reward_id = if draw_number == 1 && config.first_get != 0 {
                config.first_get
            } else {
                select_gacha_reward(&config, guaranteed, tables)
                    .ok_or(GachaError::MissingReward(gacha_id))?
            };
            let mut reward = self
                .apply_gacha_reward(reward_id, config.pool_type, tables, now)
                .ok_or(GachaError::MissingReward(gacha_id))?;
            reward.idx = offset;
            events.extend(
                self.completed_achievements_since(&previous_achievements, tables, now)
                    .into_iter()
                    .map(GachaEvent::Achievement),
            );
            previous_achievements = self.achievements(tables, now);
            for archive in self.newly_unlocked_archives_since(&unlocked_archives, tables, now) {
                unlocked_archives.push(archive.id);
                events.push(GachaEvent::Archive(archive));
            }
            if reward.reward_type == 3
                && reward.duplicate == 0
                && let Some(avatar) = self.unlock_role_avatar(reward.reward, tables)
            {
                events.push(GachaEvent::ProfileAvatar(avatar));
            }
            self.gacha_logs.push(DcNetDataGachaLog {
                time: now,
                reward: reward_id,
                gacha_id,
            });
            // Live ten-pull captures reset the counter for maid results, while a
            // quality-5 partner leaves it running and does not set `hit_10`.
            let hit_10 = tables.maids.get(reward_id).is_some();
            if hit_10 {
                pity_count = 0;
            } else {
                pity_count = pity_count.saturating_add(1);
            }
            let extra_coin = tables
                .gacha_draw_types
                .get(config.pool_type)
                .map_or(0, |draw_type| draw_type.bonus_currency_amount);
            if let Some(draw_type) = tables.gacha_draw_types.get(config.pool_type)
                && extra_coin > 0
            {
                coin = self.add_item(draw_type.bonus_currency_id, extra_coin, tables);
            }
            reward_list.push(DcNetDataGachaResInfo {
                // A captured quality-6 normal-pool maid still has hit_s=false;
                // the actual featured-hit rule is not present in the client dump.
                hit_s: false,
                hit_10,
                r#type: count,
                time: now,
                gacha_id,
                reward_id: source_reward_id,
                reward_res: Some(reward),
                extra_coin,
                ..Default::default()
            });
        }
        {
            let state = &mut self.gachas[state_index];
            state.all_count = old_count.saturating_add(count);
            // Captured limited-pool draws update only all_count. The selectable
            // cumulative character reward is exposed by the normal pool (type 1).
            if config.pool_type == 1 {
                state.reward_cnt = state.reward_cnt.saturating_add(count);
                let mut needed = if state.taken_new_reward || state.reward_num > 0 {
                    tables.cultivation_constants.cumulative_draw_target
                } else {
                    tables.cultivation_constants.new_pool_cumulative_draw_target
                };
                while needed > 0 && state.reward_cnt >= needed {
                    state.reward_cnt -= needed;
                    state.reward_num = state.reward_num.saturating_add(1);
                    state.taken_new_reward = true;
                    state.had_take_reward = false;
                    needed = tables.cultivation_constants.cumulative_draw_target;
                }
            }
        }
        self.set_gacha_pity_count(config.group_id, pity_count, tables);
        let gacha_data = self.gacha_info(
            &config,
            tables,
            now,
            zone_offset,
            false,
            override_expires_at,
        );
        let battle_pass_updates =
            self.changed_battle_pass_tasks_since(&previous_battle_pass, tables, now, zone_offset);
        Ok(GachaOutcome {
            reward_list,
            gacha_data,
            remains,
            coin,
            events,
            battle_pass_updates,
        })
    }

    fn gacha_info(
        &self,
        config: &configs::tables::GachaPool,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
        include_goods: bool,
        override_expires_at: Option<i32>,
    ) -> DcNetDataGachaData {
        let state = self
            .gachas
            .iter()
            .find(|state| state.gacha_id == config.id)
            .copied();
        let mall_goods = if include_goods {
            tables
                .mall_goods_groups
                .iter()
                .filter(|group| group.mall_id == config.mall_id)
                .flat_map(|group| {
                    tables
                        .mall_goods
                        .iter()
                        .filter(move |goods| goods.goods_group_id == group.goods_group_id)
                })
                .map(|goods| DcNetDataMallGoods {
                    id: goods.id,
                    cd_time: gacha_seconds_remaining(config, now, zone_offset, override_expires_at)
                        .unwrap_or_default(),
                    bought: 0,
                })
                .collect()
        } else {
            Vec::new()
        };
        let cumulative = config.pool_type == 1;
        DcNetDataGachaData {
            gacha_id: config.id,
            remain_sec: gacha_seconds_remaining(config, now, zone_offset, override_expires_at)
                .unwrap_or_default(),
            gacha_count_10: self.gacha_pity_count(config, tables),
            reward10_id: config.ten_reward_id,
            gacha_type: config.pool_type,
            min_10: config.guaranteed_draw_interval,
            all_count: state.map_or(0, |state| state.all_count),
            reward_cnt: state
                .filter(|_| cumulative)
                .map_or(0, |state| state.reward_cnt),
            reward_num: state
                .filter(|_| cumulative)
                .map_or(0, |state| state.reward_num),
            taken_new_reward: cumulative && state.is_some_and(|state| state.taken_new_reward),
            mall_goods,
            buffers_items: Vec::new(),
            had_take_reward: cumulative && state.is_some_and(|state| state.had_take_reward),
        }
    }

    fn gacha_pity_count(&self, config: &configs::tables::GachaPool, tables: &GameTables) -> i32 {
        self.gachas
            .iter()
            .filter(|state| {
                tables
                    .gacha_pools
                    .get(state.gacha_id)
                    .is_some_and(|pool| pool.group_id == config.group_id)
            })
            .max_by_key(|state| state.gacha_id)
            .map_or(0, |state| state.gacha_count_10)
    }

    fn set_gacha_pity_count(&mut self, group_id: i32, count: i32, tables: &GameTables) {
        for state in &mut self.gachas {
            if tables
                .gacha_pools
                .get(state.gacha_id)
                .is_some_and(|pool| pool.group_id == group_id)
            {
                state.gacha_count_10 = count;
            }
        }
    }

    pub fn gacha_logs(&self, group_id: i32, tables: &GameTables) -> Vec<DcNetDataGachaLog> {
        self.gacha_logs
            .iter()
            .rev()
            .filter(|log| {
                tables
                    .gacha_pools
                    .get(log.gacha_id)
                    .is_some_and(|config| config.group_id == group_id)
            })
            .copied()
            .collect()
    }

    pub fn claim_gacha_cumulative(
        &mut self,
        reward_id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<GachaClaimOutcome, GachaError> {
        if !tables
            .cultivation_constants
            .cumulative_draw_rewards
            .contains(&reward_id)
            || tables.maids.get(reward_id).is_none()
        {
            return Err(GachaError::InvalidCumulativeReward(reward_id));
        }
        let state_index = self
            .gachas
            .iter()
            .position(|state| {
                state.reward_num > 0
                    && tables
                        .gacha_pools
                        .get(state.gacha_id)
                        .is_some_and(|pool| pool.pool_type == 1)
            })
            .ok_or(GachaError::CumulativeRewardUnavailable)?;
        let gacha_id = self.gachas[state_index].gacha_id;
        let config = tables
            .gacha_pools
            .get(gacha_id)
            .cloned()
            .ok_or(GachaError::Unknown(gacha_id))?;

        let previous_achievements = self.achievements(tables, now);
        let unlocked_archives = self.unlocked_archive_ids(tables, now);
        let mut reward = DcNetDataTakeRewardRes::default();
        self.add_reward(reward_id, 1, tables, now, &mut reward);
        let mut events = self
            .completed_achievements_since(&previous_achievements, tables, now)
            .into_iter()
            .map(GachaEvent::Achievement)
            .collect::<Vec<_>>();
        events.extend(
            self.newly_unlocked_archives_since(&unlocked_archives, tables, now)
                .into_iter()
                .map(GachaEvent::Archive),
        );
        events.extend(
            reward
                .profileavatar_rewards
                .drain(..)
                .map(GachaEvent::ProfileAvatar),
        );
        let state = &mut self.gachas[state_index];
        state.reward_num -= 1;
        state.taken_new_reward = true;
        state.had_take_reward = state.reward_num == 0;
        let gacha_data = self.gacha_info(&config, tables, now, zone_offset, true, None);
        Ok(GachaClaimOutcome {
            reward,
            gacha_data,
            events,
        })
    }

    pub(super) fn apply_gacha_reward(
        &mut self,
        id: i32,
        pool_type: i32,
        tables: &GameTables,
        now: i32,
    ) -> Option<DcNetDataRewardsRes> {
        if let Some(maid) = tables.maids.get(id) {
            if self.owns_role(id) {
                let convert = self.role_duplicate_reward(id, Some(pool_type), tables)?;
                let item = self.add_item(convert.key, convert.value, tables)?;
                return Some(DcNetDataRewardsRes {
                    user_dat_id: item.user_item_id,
                    reward: id,
                    // Captured duplicate maids use type 2 and identify the
                    // converted fragment/item through `duplicate`.
                    reward_type: 2,
                    amount: item.amount,
                    duplicate: item.item_id,
                    quality: maid.quality,
                    ..Default::default()
                });
            }
            let role = gacha_role(self.uid, id, tables)?;
            let user_dat_id = role.role_basic_info.as_ref()?.user_role_id;
            self.roles.push(role);
            self.record_current_archive_unlocks(tables, now);
            return Some(DcNetDataRewardsRes {
                user_dat_id,
                reward: id,
                reward_type: 3,
                amount: 1,
                quality: maid.quality,
                ..Default::default()
            });
        }
        if let Some(partner) = tables.partners.get(id) {
            let user_dat_id = self.next_partner_instance_id(id);
            self.partners.push(DcNetDataPartner {
                id: user_dat_id,
                lv: 1,
                partner_id: id,
                quality: partner.quality,
                reson_lv: 1,
                skill_lv: 1,
                creat_at: i64::from(now),
                ..Default::default()
            });
            self.record_current_archive_unlocks(tables, now);
            return Some(DcNetDataRewardsRes {
                user_dat_id,
                reward: id,
                // Captured partner pulls use reward type 8.
                reward_type: 8,
                amount: 1,
                quality: partner.quality,
                ..Default::default()
            });
        }
        if let Some(equip) = tables.equipment.get(id) {
            let offset = self
                .equips
                .iter()
                .filter(|current| current.equip_id == id)
                .count() as i64;
            let user_dat_id = make_entity_id(self.uid, id).wrapping_add(offset);
            self.equips.push(DcNetDataEquip {
                equip_id: id,
                user_equip_id: user_dat_id,
                pos: equip.slot,
                main_words_id: 1,
                quality: equip.quality,
                tmp_word_idx: -1,
                creat_at: i64::from(now),
                ..Default::default()
            });
            self.record_current_archive_unlocks(tables, now);
            return Some(DcNetDataRewardsRes {
                user_dat_id,
                reward: id,
                reward_type: 4,
                amount: 1,
                quality: equip.quality,
                ..Default::default()
            });
        }
        let item = self.add_item(id, 1, tables)?;
        Some(DcNetDataRewardsRes {
            user_dat_id: item.user_item_id,
            reward: id,
            reward_type: 1,
            amount: item.amount,
            quality: item.quality,
            ..Default::default()
        })
    }
}

fn banner_override_expiration(overrides: &[(i32, i32)], id: i32, now: i32) -> Option<i32> {
    overrides.iter().find_map(|&(banner_id, expires_at)| {
        (banner_id == id && expires_at > now).then_some(expires_at)
    })
}
#[cfg(test)]
mod tests;
