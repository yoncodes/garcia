use std::collections::{BTreeMap, HashSet};

use configs::{GameTables, tables::TablePair};

use super::*;

impl Player {
    pub fn owns_partner(&self, id: i32) -> bool {
        self.partners.iter().any(|partner| partner.partner_id == id)
    }

    pub(super) fn next_partner_instance_id(&self, partner_id: i32) -> i64 {
        let base = make_entity_id(self.uid, partner_id);
        (0..=self.partners.len())
            .map(|offset| base + offset as i64)
            .find(|candidate| !self.partners.iter().any(|partner| partner.id == *candidate))
            .expect("an unused partner instance ID exists within len + 1 candidates")
    }

    pub fn grant_partner(
        &mut self,
        id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
    ) -> DcNetDataTakeRewardRes {
        let mut rewards = DcNetDataTakeRewardRes::default();
        if tables.partners.get(id).is_some() {
            self.add_reward(id, amount, tables, now, &mut rewards);
        }
        rewards
    }

    pub fn grant_all_partners(&mut self, tables: &GameTables, now: i32) -> DcNetDataTakeRewardRes {
        let missing = tables
            .partners
            .rows
            .iter()
            .map(|partner| partner.id)
            .filter(|id| !self.owns_partner(*id))
            .collect::<Vec<_>>();
        let mut rewards = DcNetDataTakeRewardRes::default();
        for id in missing {
            rewards
                .partner_rewards
                .extend(self.grant_partner(id, 1, tables, now).partner_rewards);
        }
        rewards
    }

    pub fn max_partner(
        &mut self,
        partner_table_id: i32,
        tables: &GameTables,
    ) -> Option<(DcNetDataPartner, bool)> {
        let config = tables.partners.get(partner_table_id)?;
        let equipped = self
            .roles
            .iter()
            .filter_map(|role| role.role_basic_info.as_ref())
            .map(|role| role.user_partner_id)
            .collect::<HashSet<_>>();
        let partner_index = self
            .partners
            .iter()
            .enumerate()
            .filter(|(_, partner)| partner.partner_id == partner_table_id)
            .max_by_key(|(_, partner)| {
                (
                    equipped.contains(&partner.id),
                    partner.reson_lv,
                    partner.brek,
                    partner.lv,
                    partner.skill_lv,
                )
            })?
            .0;
        let partner = &mut self.partners[partner_index];
        let before = *partner;
        partner.brek = tables
            .partner_ascensions_by_group
            .get(config.group_id)
            .into_iter()
            .flatten()
            .map(|row| row.rank)
            .max()
            .unwrap_or(partner.brek);
        partner.lv = tables
            .partner_levels_by_quality
            .get(config.quality)
            .into_iter()
            .flatten()
            .map(|row| row.level)
            .max()
            .unwrap_or(partner.lv);
        partner.exp = 0;
        partner.skill_lv = tables
            .partner_skills_by_group
            .get(config.skill_cost_group)
            .into_iter()
            .flatten()
            .map(|row| row.level)
            .max()
            .unwrap_or(partner.skill_lv);
        partner.reson_lv = tables
            .partner_resonance_costs
            .rows
            .iter()
            .map(|row| row.resonance_level)
            .max()
            .unwrap_or(partner.reson_lv);
        Some((*partner, *partner != before))
    }

    pub fn set_role_skin(
        &mut self,
        role_id: i32,
        skin_id: i32,
        tables: &GameTables,
    ) -> Result<(), RoleMutationError> {
        let role_index = self.role_index(role_id)?;
        if skin_id != 0
            && (!self.skins.contains(&skin_id)
                || tables
                    .maid_skins
                    .get(skin_id)
                    .is_none_or(|skin| skin.maid_id != role_id))
        {
            return Err(RoleMutationError::LockedSkin { role_id, skin_id });
        }
        self.roles[role_index]
            .role_basic_info
            .as_mut()
            .unwrap()
            .skin_id = skin_id;
        Ok(())
    }

    pub fn change_partner(
        &mut self,
        partner_id: i64,
        role_id: i32,
    ) -> Result<PartnerChangeOutcome, RoleMutationError> {
        let partner = self.partner(partner_id)?;
        let role_index = self.role_index(role_id)?;
        let unset_index = self.roles.iter().position(|role| {
            role.role_basic_info.as_ref().is_some_and(|role| {
                role.game_role_id != role_id && role.user_partner_id == partner_id
            })
        });
        let unset_role_info = unset_index.and_then(|index| {
            let role = self.roles[index].role_basic_info.as_mut()?;
            role.user_partner_id = 0;
            Some(role.clone())
        });
        let role_info = self.roles[role_index].role_basic_info.as_mut().unwrap();
        role_info.user_partner_id = partner_id;
        Ok(PartnerChangeOutcome {
            partner,
            role_info: role_info.clone(),
            unset_role_info,
        })
    }

    pub fn unset_partner(
        &mut self,
        partner_id: i64,
    ) -> Result<PartnerUnsetOutcome, RoleMutationError> {
        let partner = self.partner(partner_id)?;
        let role = self
            .roles
            .iter_mut()
            .filter_map(|role| role.role_basic_info.as_mut())
            .find(|role| role.user_partner_id == partner_id)
            .ok_or(RoleMutationError::PartnerNotEquipped(partner_id))?;
        role.user_partner_id = 0;
        Ok(PartnerUnsetOutcome {
            partner,
            role_info: role.clone(),
        })
    }

    pub fn swap_partners(
        &mut self,
        partner_id1: i64,
        partner_id2: i64,
    ) -> Result<PartnerSwapOutcome, RoleMutationError> {
        if partner_id1 == partner_id2 {
            return Err(RoleMutationError::SamePartner);
        }
        self.partner(partner_id1)?;
        self.partner(partner_id2)?;
        let role_index1 = self
            .partner_role_index(partner_id1)
            .ok_or(RoleMutationError::PartnerNotEquipped(partner_id1))?;
        let role_index2 = self
            .partner_role_index(partner_id2)
            .ok_or(RoleMutationError::PartnerNotEquipped(partner_id2))?;
        let (first, second) = get_two_mut(&mut self.roles, role_index1, role_index2);
        let role_info1 = first.role_basic_info.as_mut().unwrap();
        let role_info2 = second.role_basic_info.as_mut().unwrap();
        role_info1.user_partner_id = partner_id2;
        role_info2.user_partner_id = partner_id1;
        Ok(PartnerSwapOutcome {
            role_info1: role_info1.clone(),
            role_info2: role_info2.clone(),
        })
    }

    pub fn lock_partner(
        &mut self,
        partner_id: i64,
        locked: i32,
    ) -> Result<DcNetDataPartner, RoleMutationError> {
        if !matches!(locked, 0 | 1) {
            return Err(RoleMutationError::InvalidPartnerLock(locked));
        }
        let partner = self
            .partners
            .iter_mut()
            .find(|partner| partner.id == partner_id)
            .ok_or(RoleMutationError::UnknownPartner(partner_id))?;
        partner.locked = locked;
        Ok(*partner)
    }

    pub fn level_up_partner(
        &mut self,
        partner_id: i64,
        materials: &[DcNetDataItem],
        tables: &GameTables,
    ) -> Result<PartnerLevelOutcome, PartnerProgressError> {
        let partner_index = self.partner_progress_index(partner_id)?;
        let partner = self.partners[partner_index];
        let config = tables
            .partners
            .get(partner.partner_id)
            .ok_or(PartnerProgressError::MissingConfig(partner.partner_id))?;
        let level_limit = partner_level_limit(config.quality, partner.brek, tables);
        if partner.lv >= level_limit {
            return Err(PartnerProgressError::LevelCap(partner_id));
        }

        let mut costs = BTreeMap::new();
        let mut added_exp = 0i32;
        let mut exp_item_id = None;
        for material in materials {
            if material.amount < 0
                || !tables
                    .cultivation_constants
                    .partner_exp_items
                    .contains(&material.item_id)
            {
                return Err(PartnerProgressError::InvalidMaterial(material.item_id));
            }
            if material.amount == 0 {
                continue;
            }
            let effect = tables
                .items
                .get(material.item_id)
                .and_then(|item| item.effect.as_ref())
                .ok_or(PartnerProgressError::InvalidMaterial(material.item_id))?;
            if exp_item_id.is_some_and(|item_id| item_id != effect.key) {
                return Err(PartnerProgressError::InvalidMaterial(material.item_id));
            }
            exp_item_id = Some(effect.key);
            *costs.entry(material.item_id).or_default() += material.amount;
            added_exp = added_exp.saturating_add(effect.value.saturating_mul(material.amount));
        }
        if costs.is_empty() {
            return Err(PartnerProgressError::InvalidMaterial(0));
        }
        let exp_item_id = exp_item_id.unwrap();
        let gold_cost =
            (added_exp as f32 * tables.cultivation_constants.partner_exp_gold_rate) as i32;
        *costs
            .entry(tables.cultivation_constants.coin_item_id)
            .or_default() += gold_cost;
        let cost_remain = consume_partner_costs(&mut self.items, &costs)?;

        let partner = &mut self.partners[partner_index];
        let mut exp = partner.exp.saturating_add(added_exp);
        while partner.lv < level_limit {
            let Some(level) = tables
                .partner_levels_by_quality
                .get(config.quality)
                .and_then(|mut levels| levels.find(|row| row.level == partner.lv))
            else {
                break;
            };
            if exp < level.required_exp {
                break;
            }
            exp -= level.required_exp;
            partner.lv += 1;
        }
        if partner.lv == level_limit {
            exp = 0;
        }
        partner.exp = exp;
        Ok(PartnerLevelOutcome {
            partner: *partner,
            cost_remain,
            remain: vec![DcNetDataItem {
                item_id: exp_item_id,
                amount: exp,
                quality: tables.items.get(exp_item_id).map_or(0, |item| item.quality),
                ..Default::default()
            }],
        })
    }

    pub fn break_partner(
        &mut self,
        partner_id: i64,
        tables: &GameTables,
    ) -> Result<(DcNetDataPartner, Vec<DcNetDataItem>), PartnerProgressError> {
        let partner_index = self.partner_progress_index(partner_id)?;
        let partner = self.partners[partner_index];
        let config = tables
            .partners
            .get(partner.partner_id)
            .ok_or(PartnerProgressError::MissingConfig(partner.partner_id))?;
        if partner.lv < partner_level_limit(config.quality, partner.brek, tables) {
            return Err(PartnerProgressError::BreakLevelRequired(partner_id));
        }
        let row = tables
            .partner_ascensions_by_group
            .get(config.group_id)
            .and_then(|mut rows| rows.find(|row| row.rank == partner.brek))
            .filter(|row| !row.cost.is_empty())
            .ok_or(PartnerProgressError::BreakCap(partner_id))?;
        let world_level = self.world_level(tables);
        if world_level < row.required_world_level {
            return Err(PartnerProgressError::WorldLevelRequired {
                needed: row.required_world_level,
                current: world_level,
            });
        }
        let costs = aggregate_costs(&row.cost);
        let remains = consume_partner_costs(&mut self.items, &costs)?;
        self.partners[partner_index].brek += 1;
        Ok((self.partners[partner_index], remains))
    }

    pub fn upgrade_partner_skill(
        &mut self,
        partner_id: i64,
        tables: &GameTables,
    ) -> Result<(DcNetDataPartner, Vec<DcNetDataItem>), PartnerProgressError> {
        let partner_index = self.partner_progress_index(partner_id)?;
        let partner = self.partners[partner_index];
        let config = tables
            .partners
            .get(partner.partner_id)
            .ok_or(PartnerProgressError::MissingConfig(partner.partner_id))?;
        let row = tables
            .partner_skills_by_group
            .get(config.skill_cost_group)
            .and_then(|mut rows| rows.find(|row| row.level == partner.skill_lv))
            .filter(|row| !row.item_cost.is_empty())
            .ok_or(PartnerProgressError::SkillCap(partner_id))?;
        let costs = aggregate_costs(&row.item_cost);
        let remains = consume_partner_costs(&mut self.items, &costs)?;
        self.partners[partner_index].skill_lv += 1;
        Ok((self.partners[partner_index], remains))
    }

    pub fn resonate_partner(
        &mut self,
        partner_id: i64,
        cost_partner_ids: &[i64],
        tables: &GameTables,
    ) -> Result<PartnerResonanceOutcome, PartnerProgressError> {
        let partner_index = self.partner_progress_index(partner_id)?;
        let target = self.partners[partner_index];
        let max_level = tables
            .partner_resonance_costs
            .rows
            .iter()
            .map(|row| row.resonance_level)
            .max()
            .unwrap_or(1);
        if target.reson_lv >= max_level {
            return Err(PartnerProgressError::ResonanceCap(partner_id));
        }

        let mut seen = HashSet::new();
        let mut gained = 0i32;
        for &cost_id in cost_partner_ids {
            if cost_id == partner_id || !seen.insert(cost_id) {
                return Err(PartnerProgressError::InvalidResonancePartner(cost_id));
            }
            let cost = self
                .partners
                .iter()
                .find(|partner| partner.id == cost_id)
                .ok_or(PartnerProgressError::InvalidResonancePartner(cost_id))?;
            if cost.partner_id != target.partner_id {
                return Err(PartnerProgressError::InvalidResonancePartner(cost_id));
            }
            if cost.locked != 0 {
                return Err(PartnerProgressError::LockedResonancePartner(cost_id));
            }
            if self.partner_role_index(cost_id).is_some() {
                return Err(PartnerProgressError::EquippedResonancePartner(cost_id));
            }
            gained = gained.saturating_add(cost.reson_lv);
        }
        if gained == 0 {
            return Err(PartnerProgressError::InvalidResonancePartner(0));
        }
        let new_level = target.reson_lv.saturating_add(gained).min(max_level);
        let mut costs = BTreeMap::new();
        for level in target.reson_lv..new_level {
            if let Some(cost) = tables
                .partner_resonance_costs
                .get(level)
                .and_then(|row| row.cost.as_ref())
            {
                *costs.entry(cost.key).or_default() += cost.value;
            }
        }
        let remains = consume_partner_costs(&mut self.items, &costs)?;
        self.partners.retain(|partner| !seen.contains(&partner.id));
        let partner = self
            .partners
            .iter_mut()
            .find(|partner| partner.id == partner_id)
            .unwrap();
        partner.reson_lv = new_level;
        Ok(PartnerResonanceOutcome {
            partner: *partner,
            consumed_partners: cost_partner_ids.to_vec(),
            remains,
        })
    }

    fn partner_progress_index(&self, partner_id: i64) -> Result<usize, PartnerProgressError> {
        self.partners
            .iter()
            .position(|partner| partner.id == partner_id)
            .ok_or(PartnerProgressError::UnknownPartner(partner_id))
    }

    fn partner(&self, partner_id: i64) -> Result<DcNetDataPartner, RoleMutationError> {
        self.partners
            .iter()
            .find(|partner| partner.id == partner_id)
            .copied()
            .ok_or(RoleMutationError::UnknownPartner(partner_id))
    }

    fn role_index(&self, role_id: i32) -> Result<usize, RoleMutationError> {
        self.roles
            .iter()
            .position(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|role| role.game_role_id == role_id)
            })
            .ok_or(RoleMutationError::UnknownRole(role_id))
    }

    fn partner_role_index(&self, partner_id: i64) -> Option<usize> {
        self.roles.iter().position(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|role| role.user_partner_id == partner_id)
        })
    }
}

fn partner_level_limit(quality: i32, rank: i32, tables: &GameTables) -> i32 {
    let mut break_rank = 0;
    let mut last_level = 0;
    let Some(levels) = tables.partner_levels_by_quality.get(quality) else {
        return 0;
    };
    for level in levels {
        last_level = level.level;
        if level.breakpoint == 1 {
            if break_rank == rank {
                return level.level;
            }
            break_rank += 1;
        }
    }
    last_level
}

fn aggregate_costs(costs: &[TablePair<i32>]) -> BTreeMap<i32, i32> {
    let mut result = BTreeMap::new();
    for cost in costs {
        *result.entry(cost.key).or_default() += cost.value;
    }
    result
}

fn consume_partner_costs(
    inventory: &mut [DcNetDataItem],
    costs: &BTreeMap<i32, i32>,
) -> Result<Vec<DcNetDataItem>, PartnerProgressError> {
    consume_item_costs(inventory, costs, |item_id, needed, available| {
        PartnerProgressError::InsufficientItem {
            item_id,
            needed,
            available,
        }
    })
}

fn get_two_mut<T>(values: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    debug_assert_ne!(first, second);
    if first < second {
        let (left, right) = values.split_at_mut(second);
        (&mut left[first], &mut right[0])
    } else {
        let (left, right) = values.split_at_mut(first);
        (&mut right[0], &mut left[second])
    }
}
#[cfg(test)]
mod tests;
