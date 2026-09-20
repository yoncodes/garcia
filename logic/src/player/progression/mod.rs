use super::*;

mod advancement;
pub use advancement::ProgressionOutcome;

impl Player {
    pub fn grant_role(&mut self, id: i32, tables: &GameTables, now: i32) -> DcNetDataTakeRewardRes {
        let mut rewards = DcNetDataTakeRewardRes::default();
        if tables.is_playable_maid(id) {
            self.add_reward(id, 1, tables, now, &mut rewards);
        }
        rewards
    }

    pub(super) fn role_duplicate_reward(
        &self,
        role_id: i32,
        pool_type: Option<i32>,
        tables: &GameTables,
    ) -> Option<configs::tables::TablePair<i32>> {
        let maid = tables.maids.get(role_id)?;
        let resonance = self.roles.iter().find_map(|role| {
            role.role_basic_info
                .as_ref()
                .filter(|role| role.game_role_id == role_id)
                .map(|role| role.maid_qua)
        })?;
        let fragments = self
            .items
            .iter()
            .find(|item| item.item_id == maid.quality_break.key)
            .map_or(0, |item| item.amount);
        if resonance.saturating_add(fragments) < tables.cultivation_constants.max_role_resonance {
            return Some(maid.duplicate_reward.clone());
        }

        let mut conversions = tables.gacha_rewards.iter().filter(|reward| {
            reward.quality == maid.quality
                && reward.converts_duplicates != 0
                && pool_type.is_none_or(|pool_type| reward.pool_type == pool_type)
        });
        let conversion = conversions.next()?.convert.clone()?;
        conversions
            .all(|reward| {
                reward.convert.as_ref().is_some_and(|candidate| {
                    candidate.key == conversion.key && candidate.value == conversion.value
                })
            })
            .then_some(conversion)
    }

    pub fn grant_all_roles(&mut self, tables: &GameTables, now: i32) -> DcNetDataTakeRewardRes {
        let missing = tables
            .maids
            .rows
            .iter()
            .map(|maid| maid.id)
            .filter(|id| tables.is_playable_maid(*id))
            .filter(|id| !self.owns_role(*id))
            .collect::<Vec<_>>();
        let mut rewards = DcNetDataTakeRewardRes::default();
        for id in missing {
            let granted = self.grant_role(id, tables, now);
            rewards.role_rewards.extend(granted.role_rewards);
            rewards
                .profileavatar_rewards
                .extend(granted.profileavatar_rewards);
        }
        rewards
    }

    pub fn max_role(
        &mut self,
        role_id: i32,
        tables: &GameTables,
    ) -> Result<RoleMaxOutcome, RoleMutationError> {
        let role_index = self.role_progress_index(role_id)?;
        let maid = tables
            .maids
            .get(role_id)
            .ok_or(RoleMutationError::UnknownRole(role_id))?;
        let max_rank = tables
            .maid_ranks_by_maid
            .get(role_id)
            .into_iter()
            .flatten()
            .map(|rank| rank.rank)
            .max()
            .unwrap_or_default();
        let max_level = role_level_limit(max_rank, tables);
        let before = self.roles[role_index].clone();
        let role = &mut self.roles[role_index];
        let info = role.role_basic_info.as_mut().unwrap();
        info.level = max_level;
        info.exp = 0;
        info.position = max_rank;
        info.maid_qua = tables.cultivation_constants.max_role_resonance;
        for talent in &mut role.talents {
            talent.lv = tables
                .maid_talent_levels
                .iter()
                .filter(|level| level.quality == maid.quality && level.position == talent.position)
                .map(|level| level.level)
                .max()
                .unwrap_or(talent.lv);
        }
        let role = self.roles[role_index].clone();
        let changed = role != before;
        let namecard = self.unlock_maxed_role_namecard(role_id, tables);
        Ok(RoleMaxOutcome {
            role,
            namecard,
            changed,
        })
    }

    pub fn start_port(&mut self, id: i32, tables: &GameTables) -> Result<(), PortError> {
        tables
            .main_story_ports
            .get(id)
            .ok_or(PortError::Unknown(id))?;
        self.gameplay_id = id;
        Ok(())
    }

    pub fn settle_port(
        &mut self,
        settlement: &DcNetDataParamsSettlement,
        tables: &GameTables,
    ) -> Result<(DcNetDataPort, Vec<RoleInfo>), PortError> {
        if self.gameplay_id != settlement.id {
            return Err(PortError::NotActive {
                expected: self.gameplay_id,
                actual: settlement.id,
            });
        }
        let config = tables
            .main_story_ports
            .get(settlement.id)
            .ok_or(PortError::Unknown(settlement.id))?
            .clone();
        if let Some(monster) = settlement.monsters.iter().find(|monster| {
            !tables
                .monsters
                .rows
                .iter()
                .any(|config| config.id_crc == monster.enemy_hash)
        }) {
            return Err(PortError::UnknownMonsterCrc(monster.enemy_hash));
        }
        if !settlement.monsters.is_empty() {
            let manual = if let Some(index) = self
                .monster_manuals
                .iter()
                .position(|manual| manual.port_id == settlement.id)
            {
                &mut self.monster_manuals[index]
            } else {
                self.monster_manuals.push(DcNetDataMonsterManual4Port {
                    port_id: settlement.id,
                    mons: Vec::new(),
                });
                self.monster_manuals.last_mut().unwrap()
            };
            for monster in &settlement.monsters {
                if !manual.mons.contains(&monster.enemy_hash) {
                    manual.mons.push(monster.enemy_hash);
                }
            }
        }
        let index = self
            .ports
            .iter()
            .position(|port| port.id == settlement.id)
            .unwrap_or_else(|| {
                self.ports.push(DcNetDataPort {
                    id: settlement.id,
                    port_id: config.port_id,
                    ..Default::default()
                });
                self.ports.len() - 1
            });
        let port = &mut self.ports[index];
        port.all_cnt = port.all_cnt.saturating_add(1);
        if settlement.is_completed != 0 {
            port.pass_cnt = port.pass_cnt.saturating_add(1);
            port.is_c = true;
        }
        for (index, complete) in settlement.stars.iter().take(3).enumerate() {
            if *complete {
                match index {
                    0 => port.s1 = 1,
                    1 => port.s2 = 1,
                    2 => port.s3 = 1,
                    _ => unreachable!(),
                }
            }
        }
        let port = *port;

        let roles = if settlement.is_completed != 0 {
            config.maid_exp.map_or_else(Vec::new, |reward| {
                settlement
                    .role_data
                    .iter()
                    .filter_map(|role| self.add_role_exp(role.game_role_id, &reward, tables))
                    .collect()
            })
        } else {
            Vec::new()
        };
        Ok((port, roles))
    }

    fn add_role_exp(
        &mut self,
        role_id: i32,
        reward: &configs::tables::TablePair<i32>,
        tables: &GameTables,
    ) -> Option<RoleInfo> {
        let role = self.roles.iter_mut().find_map(|role| {
            role.role_basic_info
                .as_mut()
                .filter(|info| info.game_role_id == role_id)
        })?;
        let old_level = role.level;
        let level_limit = role_level_limit(role.position, tables);
        if role.level < level_limit {
            role.exp = role.exp.saturating_add(reward.value);
        }
        while role.level < level_limit
            && let Some(level) = tables.maid_levels.get(role.level)
        {
            if role.exp < level.exp {
                break;
            }
            role.exp -= level.exp;
            role.level += 1;
        }
        if role.level == level_limit {
            role.exp = 0;
        }
        let quality = tables.items.get(reward.key).map_or(0, |item| item.quality);
        Some(RoleInfo {
            role_id,
            lv_up_data: Some(DcNetDataAttrUpData {
                attr_lv: role.level,
                attr_exp: role.exp,
                add_exp: reward.value,
                items: vec![DcNetDataItem {
                    item_id: reward.key,
                    amount: role.exp,
                    quality,
                    ..Default::default()
                }],
                is_lv_up: role.level > old_level,
            }),
            fetter_up_data: None,
        })
    }

    pub fn owns_role(&self, id: i32) -> bool {
        self.roles.iter().any(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|info| info.game_role_id == id)
        })
    }

    pub fn level_up_role(
        &mut self,
        role_id: i32,
        materials: &[DcNetDataUseItem],
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<RoleLevelOutcome, RoleMutationError> {
        let previous_achievements = self.achievements(tables, now);
        let previous_seven_day = self.seven_day_activity_status(tables, now).0;
        let role_index = self
            .roles
            .iter()
            .position(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|role| role.game_role_id == role_id)
            })
            .ok_or(RoleMutationError::UnknownRole(role_id))?;
        let role = self.roles[role_index].role_basic_info.as_ref().unwrap();
        let level_limit = role_level_limit(role.position, tables);
        if role.level >= level_limit {
            return Err(RoleMutationError::LevelCap(role_id));
        }

        let mut selected = BTreeMap::<i32, i32>::new();
        let mut added_exp = 0i32;
        let mut exp_item_id = None;
        for material in materials {
            if material.amount <= 0 {
                return Err(RoleMutationError::InvalidMaterial(material.item_id));
            }
            if !tables
                .cultivation_constants
                .role_exp_items
                .contains(&material.item_id)
            {
                return Err(RoleMutationError::InvalidMaterial(material.item_id));
            }
            let effect = tables
                .items
                .get(material.item_id)
                .and_then(|item| item.effect.as_ref())
                .ok_or(RoleMutationError::InvalidMaterial(material.item_id))?;
            if exp_item_id.is_some_and(|item_id| item_id != effect.key) {
                return Err(RoleMutationError::InvalidMaterial(material.item_id));
            }
            exp_item_id = Some(effect.key);
            let amount = selected.entry(material.item_id).or_default();
            *amount = amount.saturating_add(material.amount);
            added_exp = added_exp.saturating_add(effect.value.saturating_mul(material.amount));
        }
        if selected.is_empty() {
            return Err(RoleMutationError::InvalidMaterial(0));
        }
        let exp_item_id = exp_item_id.unwrap();
        for (item_id, needed) in &selected {
            let available = self
                .items
                .iter()
                .find(|item| item.item_id == *item_id)
                .map_or(0, |item| item.amount);
            if available < *needed {
                return Err(RoleMutationError::InsufficientMaterial {
                    item_id: *item_id,
                    needed: *needed,
                    available,
                });
            }
        }
        let gold_cost = (added_exp as f32 * tables.cultivation_constants.role_exp_gold_rate) as i32;
        let coin_item_id = tables.cultivation_constants.coin_item_id;
        let gold_available = self
            .items
            .iter()
            .find(|item| item.item_id == coin_item_id)
            .map_or(0, |item| item.amount);
        if gold_available < gold_cost {
            return Err(RoleMutationError::InsufficientGold {
                needed: gold_cost,
                available: gold_available,
            });
        }

        let previous_battle_pass = self.battle_pass_task_snapshot(tables, now, zone_offset);
        let mut remain = Vec::with_capacity(selected.len() + 1);
        for (item_id, amount) in selected {
            let item = self
                .items
                .iter_mut()
                .find(|item| item.item_id == item_id)
                .unwrap();
            item.amount -= amount;
            remain.push(*item);
            let spent = self.item_spent.entry(item_id).or_default();
            *spent = spent.saturating_add(amount);
        }
        let gold = self
            .items
            .iter_mut()
            .find(|item| item.item_id == coin_item_id)
            .unwrap();
        gold.amount -= gold_cost;
        remain.push(*gold);
        let spent_gold = self.item_spent.entry(coin_item_id).or_default();
        *spent_gold = spent_gold.saturating_add(gold_cost);

        let role = self.roles[role_index].role_basic_info.as_mut().unwrap();
        let old_level = role.level;
        let mut exp = role.exp.saturating_add(added_exp);
        while role.level < level_limit {
            let Some(level) = tables.maid_levels.get(role.level) else {
                break;
            };
            if exp < level.exp {
                break;
            }
            exp -= level.exp;
            role.level += 1;
        }
        // Captured upgrades that reach rank 0's level-20 cap report zero
        // attr_exp and return representable overflow as EXP materials.
        let overflow = if role.level == level_limit {
            let overflow = exp;
            exp = 0;
            overflow
        } else {
            0
        };
        role.exp = exp;
        let role_level = role.level;
        let leveled_up = role_level > old_level;
        let mut exp_items = vec![DcNetDataItem {
            item_id: exp_item_id,
            amount: exp,
            quality: tables.items.get(exp_item_id).map_or(0, |item| item.quality),
            ..Default::default()
        }];
        if overflow > 0 {
            let mut remaining = overflow;
            let mut materials = tables
                .cultivation_constants
                .role_exp_items
                .iter()
                .filter_map(|&item_id| {
                    tables
                        .items
                        .get(item_id)?
                        .effect
                        .as_ref()
                        .filter(|effect| effect.key == exp_item_id && effect.value > 0)
                        .map(|effect| (effect.value, item_id))
                })
                .collect::<Vec<_>>();
            materials.sort_unstable_by(|left, right| right.cmp(left));
            for (value, item_id) in materials {
                let amount = remaining / value;
                if amount == 0 {
                    continue;
                }
                remaining %= value;
                if let Some(item) = self.add_item(item_id, amount, tables) {
                    exp_items.push(item);
                }
            }
        }
        let user_up_data = DcNetDataAttrUpData {
            attr_lv: role_level,
            attr_exp: exp,
            add_exp: added_exp,
            items: exp_items,
            is_lv_up: leveled_up,
        };
        let talents = self.roles[role_index].talents.clone();
        let achievement_updates =
            self.completed_achievements_since(&previous_achievements, tables, now);
        let seven_day_updates =
            self.completed_seven_day_activities_since(&previous_seven_day, tables, now);
        let daily_updates = if leveled_up {
            self.advance_daily_condition(16, 1, tables)
        } else {
            Vec::new()
        };
        let battle_pass_updates =
            self.changed_battle_pass_tasks_since(&previous_battle_pass, tables, now, zone_offset);
        Ok(RoleLevelOutcome {
            user_up_data,
            remain,
            talents,
            battle_pass_updates,
            achievement_updates,
            seven_day_updates,
            daily_updates,
        })
    }

    pub fn rank_up_role(
        &mut self,
        role_id: i32,
        tables: &GameTables,
    ) -> Result<RoleRankOutcome, RoleMutationError> {
        let role_index = self.role_progress_index(role_id)?;
        let role = self.roles[role_index].role_basic_info.as_ref().unwrap();
        if role.level < role_level_limit(role.position, tables) {
            return Err(RoleMutationError::RankLevelRequired(role_id));
        }
        let rank = tables
            .maid_ranks_by_maid
            .get(role_id)
            .and_then(|mut ranks| ranks.find(|rank| rank.rank == role.position))
            .filter(|rank| !rank.cost.is_empty())
            .ok_or(RoleMutationError::RankCap(role_id))?;
        let world_level = self.world_level(tables);
        if world_level < rank.required_world_level {
            return Err(RoleMutationError::RankWorldLevelRequired {
                needed: rank.required_world_level,
                current: world_level,
            });
        }
        let mut costs = BTreeMap::new();
        for cost in &rank.cost {
            *costs.entry(cost.key).or_default() += cost.value;
        }
        let items = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            RoleMutationError::InsufficientItem {
                item_id,
                needed,
                available,
            }
        })?;
        let role = self.roles[role_index].role_basic_info.as_mut().unwrap();
        role.position += 1;
        Ok(RoleRankOutcome {
            game_role_id: role_id,
            position: role.position,
            items,
            talents: self.roles[role_index].talents.clone(),
        })
    }

    pub fn resonate_role(
        &mut self,
        role_id: i32,
        tables: &GameTables,
    ) -> Result<(i32, Vec<DcNetDataItem>, Option<DcNetDataProfileCard>), RoleMutationError> {
        let role_index = self.role_progress_index(role_id)?;
        let role = self.roles[role_index].role_basic_info.as_ref().unwrap();
        if role.maid_qua >= tables.cultivation_constants.max_role_resonance {
            return Err(RoleMutationError::ResonanceCap(role_id));
        }
        let cost = tables
            .maids
            .get(role_id)
            .ok_or(RoleMutationError::UnknownRole(role_id))?
            .quality_break
            .clone();
        let costs = BTreeMap::from([(cost.key, cost.value)]);
        let remains = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            RoleMutationError::InsufficientItem {
                item_id,
                needed,
                available,
            }
        })?;
        let role = self.roles[role_index].role_basic_info.as_mut().unwrap();
        role.maid_qua += 1;
        let maid_quality = role.maid_qua;
        let namecard = self.unlock_maxed_role_namecard(role_id, tables);
        Ok((maid_quality, remains, namecard))
    }

    pub fn claim_role_rank_reward(
        &mut self,
        role_id: i32,
        level: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, RoleMutationError> {
        let role_index = self.role_progress_index(role_id)?;
        let role = self.roles[role_index].role_basic_info.as_ref().unwrap();
        let rewards = tables
            .maid_ranks_by_maid
            .get(role_id)
            .and_then(|mut ranks| ranks.find(|rank| rank.rank == level))
            .filter(|rank| !rank.award.is_empty())
            .map(|rank| rank.award.clone())
            .ok_or(RoleMutationError::UnknownRank { role_id, level })?;
        if level >= role.position {
            return Err(RoleMutationError::RankRewardLocked { role_id, level });
        }
        if role.awards.contains(&level) {
            return Err(RoleMutationError::RankRewardClaimed { role_id, level });
        }
        self.roles[role_index]
            .role_basic_info
            .as_mut()
            .unwrap()
            .awards
            .push(level);
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn level_up_role_talent(
        &mut self,
        role_id: i32,
        position: i32,
        tables: &GameTables,
    ) -> Result<(DcNetDataTalent, Vec<DcNetDataItem>), RoleMutationError> {
        let role_index = self.role_progress_index(role_id)?;
        let role = &self.roles[role_index];
        let talent_index = role
            .talents
            .iter()
            .position(|talent| talent.position == position)
            .ok_or(RoleMutationError::UnknownTalent { role_id, position })?;
        let info = role.role_basic_info.as_ref().unwrap();
        let current_level = role.talents[talent_index].lv;
        let maid = tables
            .maids
            .get(role_id)
            .ok_or(RoleMutationError::UnknownRole(role_id))?;
        let next = tables
            .maid_talent_levels
            .iter()
            .find(|level| {
                level.quality == maid.quality
                    && level.position == position
                    && level.level == current_level + 1
            })
            .ok_or(RoleMutationError::TalentCap { role_id, position })?;
        let limit = next
            .unlock_limit
            .as_ref()
            .ok_or(RoleMutationError::TalentCap { role_id, position })?;
        match limit.key {
            1 if info.position < limit.value => {
                return Err(RoleMutationError::TalentRankRequired {
                    position,
                    needed: limit.value,
                    current: info.position,
                });
            }
            2 if info.level < limit.value => {
                return Err(RoleMutationError::TalentLevelRequired {
                    position,
                    needed: limit.value,
                    current: info.level,
                });
            }
            1 | 2 => {}
            key => return Err(RoleMutationError::InvalidTalentCost { role_id, key }),
        }

        let mut costs = BTreeMap::new();
        for cost in &next.cost {
            let index = usize::try_from(cost.key - 1).map_err(|_| {
                RoleMutationError::InvalidTalentCost {
                    role_id,
                    key: cost.key,
                }
            })?;
            let item_id =
                *maid
                    .talent_cost_group
                    .get(index)
                    .ok_or(RoleMutationError::InvalidTalentCost {
                        role_id,
                        key: cost.key,
                    })?;
            *costs.entry(item_id).or_default() += cost.value;
        }
        if next.cost_money > 0 {
            *costs
                .entry(tables.cultivation_constants.coin_item_id)
                .or_default() += next.cost_money;
        }
        let remain = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            RoleMutationError::InsufficientItem {
                item_id,
                needed,
                available,
            }
        })?;
        let talent = &mut self.roles[role_index].talents[talent_index];
        talent.lv += 1;
        Ok((*talent, remain))
    }

    fn role_progress_index(&self, role_id: i32) -> Result<usize, RoleMutationError> {
        self.roles
            .iter()
            .position(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|role| role.game_role_id == role_id)
            })
            .ok_or(RoleMutationError::UnknownRole(role_id))
    }
}
#[cfg(test)]
mod tests;
