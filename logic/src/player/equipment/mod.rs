use super::*;

impl Player {
    pub fn generate_relics(
        &mut self,
        relic_id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<DcNetDataEquip>, EquipmentError> {
        self.create_relics(relic_id, amount, None, None, tables, now)
    }

    pub fn generate_custom_relics(
        &mut self,
        relic_id: i32,
        amount: i32,
        main_stat: i32,
        substats: &[i32],
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<DcNetDataEquip>, EquipmentError> {
        let config = tables
            .equipment
            .get(relic_id)
            .ok_or(EquipmentError::UnknownRelic(relic_id))?;
        if config.english_name.is_empty() {
            return Err(EquipmentError::UnavailableRelic(relic_id));
        }
        if !tables
            .main_equipment_words_by_group
            .get(config.group_id)
            .is_some_and(|mut words| words.any(|word| word.id == main_stat))
        {
            return Err(EquipmentError::InvalidRelicMainStat {
                relic_id,
                word_id: main_stat,
            });
        }
        let expected = tables
            .equipment_parameters
            .get(config.quality)
            .ok_or(EquipmentError::MissingRelicParameters(relic_id))?
            .max_word_rolls() as usize;
        if substats.len() != expected {
            return Err(EquipmentError::InvalidRelicSubstatCount {
                relic_id,
                expected,
                actual: substats.len(),
            });
        }
        let mut effects = BTreeMap::<i32, i32>::new();
        for &word_id in substats {
            let word = tables
                .extra_equipment_words
                .get(word_id)
                .filter(|word| word.slot == config.slot)
                .ok_or(EquipmentError::InvalidRelicSubstat { relic_id, word_id })?;
            let count = effects.entry(word.effect.key).or_default();
            *count += 1;
            if *count > word.duplicate_limit {
                return Err(EquipmentError::RelicSubstatLimit {
                    effect_id: word.effect.key,
                    limit: word.duplicate_limit,
                });
            }
        }
        self.create_relics(
            relic_id,
            amount,
            Some(main_stat),
            Some(substats),
            tables,
            now,
        )
    }

    fn create_relics(
        &mut self,
        relic_id: i32,
        amount: i32,
        main_stat: Option<i32>,
        substats: Option<&[i32]>,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<DcNetDataEquip>, EquipmentError> {
        if amount <= 0 {
            return Err(EquipmentError::InvalidRelicAmount(amount));
        }
        let config = tables
            .equipment
            .get(relic_id)
            .ok_or(EquipmentError::UnknownRelic(relic_id))?;
        if config.english_name.is_empty() {
            return Err(EquipmentError::UnavailableRelic(relic_id));
        }
        let base = self
            .equips
            .iter()
            .filter(|relic| relic.equip_id == relic_id)
            .count() as i64;
        let mut generated = Vec::with_capacity(amount as usize);
        for offset in 0..amount {
            let mut relic = DcNetDataEquip {
                equip_id: relic_id,
                user_equip_id: make_entity_id(self.uid, relic_id)
                    .wrapping_add(base)
                    .wrapping_add(i64::from(offset)),
                pos: config.slot,
                quality: config.quality,
                tmp_word_idx: -1,
                creat_at: i64::from(now),
                ..Default::default()
            };
            if let (Some(main_stat), Some(substats)) = (main_stat, substats) {
                relic.main_words_id = main_stat;
                relic.deputy_words_id.extend_from_slice(substats);
            } else {
                initialize_equipment_words(&mut relic, self.uid, tables);
            }
            self.equips.push(relic.clone());
            generated.push(relic);
        }
        self.record_current_archive_unlocks(tables, now);
        Ok(generated)
    }

    pub fn save_equipment_group(
        &mut self,
        mut group: DcNetDataEquipsGroup,
    ) -> Result<(DcNetDataEquipsGroup, Vec<DcNetDataEquip>), EquipmentError> {
        if !self.has_role(group.game_role_id) {
            return Err(EquipmentError::UnknownRole(group.game_role_id));
        }
        if group.id < 0 {
            return Err(EquipmentError::InvalidGroupId);
        }
        self.validate_equipment_set(&group.user_equip_ids)?;

        if group.id == 0 {
            group.id = self
                .equipment_groups
                .iter()
                .map(|saved| saved.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            self.equipment_groups.push(group.clone());
        } else if let Some(saved) = self
            .equipment_groups
            .iter_mut()
            .find(|saved| saved.id == group.id)
        {
            *saved = group.clone();
        } else {
            return Err(EquipmentError::UnknownGroup(group.id));
        }

        let equips = self.recompute_equipment_group_counts();
        Ok((group, equips))
    }

    pub fn delete_equipment_group(&mut self, id: i64) -> Result<(), EquipmentError> {
        let index = self
            .equipment_groups
            .iter()
            .position(|group| group.id == id)
            .ok_or(EquipmentError::UnknownGroup(id))?;
        self.equipment_groups.remove(index);
        self.recompute_equipment_group_counts();
        Ok(())
    }

    pub fn set_equipment_group(
        &mut self,
        role_id: i32,
        equip_ids: &[i64],
    ) -> Result<Vec<DcNetDataEquipSetRes>, EquipmentError> {
        if !self.has_role(role_id) {
            return Err(EquipmentError::UnknownRole(role_id));
        }
        self.validate_equipment_set(equip_ids)?;
        equip_ids
            .iter()
            .map(|equip_id| {
                let slot = self
                    .equips
                    .iter()
                    .find(|equip| equip.user_equip_id == *equip_id)
                    .unwrap()
                    .pos;
                self.set_equipment(role_id, *equip_id, slot)
            })
            .collect()
    }

    fn validate_equipment_set(&self, equip_ids: &[i64]) -> Result<(), EquipmentError> {
        let mut slots = Vec::with_capacity(equip_ids.len());
        for equip_id in equip_ids {
            let equip = self
                .equips
                .iter()
                .find(|equip| equip.user_equip_id == *equip_id)
                .ok_or(EquipmentError::UnknownEquipment(*equip_id))?;
            if slots.contains(&equip.pos) {
                return Err(EquipmentError::DuplicateGroupSlot(equip.pos));
            }
            slots.push(equip.pos);
        }
        Ok(())
    }

    fn recompute_equipment_group_counts(&mut self) -> Vec<DcNetDataEquip> {
        let groups = &self.equipment_groups;
        self.equips
            .iter_mut()
            .filter_map(|equip| {
                let count = groups
                    .iter()
                    .filter(|group| group.user_equip_ids.contains(&equip.user_equip_id))
                    .count()
                    .min(i32::MAX as usize) as i32;
                (equip.in_group != count).then(|| {
                    equip.in_group = count;
                    equip.clone()
                })
            })
            .collect()
    }

    pub fn set_equipment(
        &mut self,
        role_id: i32,
        equip_id: i64,
        slot: i32,
    ) -> Result<DcNetDataEquipSetRes, EquipmentError> {
        if !self.has_role(role_id) {
            return Err(EquipmentError::UnknownRole(role_id));
        }
        let equip_index = self
            .equips
            .iter()
            .position(|equip| equip.user_equip_id == equip_id)
            .ok_or(EquipmentError::UnknownEquipment(equip_id))?;
        let actual = self.equips[equip_index].pos;
        if actual != slot {
            return Err(EquipmentError::WrongSlot {
                equip_id,
                actual,
                requested: slot,
            });
        }

        let dropped_list = self
            .equips
            .iter_mut()
            .enumerate()
            .filter(|(index, equip)| {
                *index != equip_index && equip.equiped_role == role_id && equip.pos == slot
            })
            .map(|(_, equip)| {
                equip.equiped_role = 0;
                equip.clone()
            })
            .collect();
        self.equips[equip_index].equiped_role = role_id;
        Ok(DcNetDataEquipSetRes {
            equip_info: Some(self.equips[equip_index].clone()),
            dropped_list,
        })
    }

    pub fn unset_equipment(
        &mut self,
        role_id: i32,
        slot: i32,
    ) -> Result<Vec<DcNetDataEquip>, EquipmentError> {
        if !self.has_role(role_id) {
            return Err(EquipmentError::UnknownRole(role_id));
        }
        Ok(self
            .equips
            .iter_mut()
            .filter(|equip| equip.equiped_role == role_id && equip.pos == slot)
            .map(|equip| {
                equip.equiped_role = 0;
                equip.clone()
            })
            .collect())
    }

    pub fn swap_equipment(
        &mut self,
        first_id: i64,
        second_id: i64,
    ) -> Result<(DcNetDataEquip, DcNetDataEquip), EquipmentError> {
        let first = self
            .equips
            .iter()
            .position(|equip| equip.user_equip_id == first_id)
            .ok_or(EquipmentError::UnknownEquipment(first_id))?;
        let second = self
            .equips
            .iter()
            .position(|equip| equip.user_equip_id == second_id)
            .ok_or(EquipmentError::UnknownEquipment(second_id))?;
        if self.equips[first].pos != self.equips[second].pos {
            return Err(EquipmentError::SwapSlotMismatch {
                first: first_id,
                second: second_id,
            });
        }
        if first == second {
            let equip = self.equips[first].clone();
            return Ok((equip.clone(), equip));
        }
        let first_role = self.equips[first].equiped_role;
        let second_role = self.equips[second].equiped_role;
        self.equips[first].equiped_role = second_role;
        self.equips[second].equiped_role = first_role;
        Ok((self.equips[first].clone(), self.equips[second].clone()))
    }

    pub fn lock_equipment(&mut self, equip_id: i64, state: i32) -> Result<(), EquipmentError> {
        if !matches!(state, 0 | 1) {
            return Err(EquipmentError::InvalidLockState);
        }
        let equip = self
            .equips
            .iter_mut()
            .find(|equip| equip.user_equip_id == equip_id)
            .ok_or(EquipmentError::UnknownEquipment(equip_id))?;
        equip.locked = state;
        Ok(())
    }

    pub fn level_up_equipment(
        &mut self,
        equip_id: i64,
        materials: &[DcNetDataUseItem],
        tables: &GameTables,
    ) -> Result<(DcNetDataEquip, Vec<DcNetDataItem>, Vec<DcNetDataItem>), EquipmentError> {
        let equip_index = self
            .equips
            .iter()
            .position(|equip| equip.user_equip_id == equip_id)
            .ok_or(EquipmentError::UnknownEquipment(equip_id))?;
        let equip = &self.equips[equip_index];
        let old_level = equip.level;
        let quality = tables
            .equipment
            .get(equip.equip_id)
            .map(|config| config.quality)
            .unwrap_or(equip.quality);
        let max_level = tables
            .equipment_parameters
            .get(quality)
            .map(|config| config.max_level)
            .ok_or(EquipmentError::MissingLevelConfig {
                equip_id: equip.equip_id,
            })?;
        if equip.level >= max_level {
            return Err(EquipmentError::LevelCap(equip_id));
        }

        let mut selected = BTreeMap::<i32, (i64, i32)>::new();
        let mut added_exp = 0i32;
        for material in materials {
            if material.amount <= 0
                || !tables
                    .cultivation_constants
                    .equipment_exp_items
                    .contains(&material.item_id)
            {
                return Err(EquipmentError::InvalidMaterial(material.item_id));
            }
            let effect = tables
                .items
                .get(material.item_id)
                .and_then(|item| item.effect.as_ref())
                .filter(|effect| effect.value > 0)
                .ok_or(EquipmentError::InvalidMaterial(material.item_id))?;
            let selected_item = selected
                .entry(material.item_id)
                .or_insert((material.user_item_id, 0));
            if selected_item.0 != material.user_item_id {
                return Err(EquipmentError::WrongItemInstance {
                    item_id: material.item_id,
                });
            }
            selected_item.1 = selected_item.1.saturating_add(material.amount);
            added_exp = added_exp.saturating_add(effect.value.saturating_mul(material.amount));
        }
        if selected.is_empty() {
            return Err(EquipmentError::InvalidMaterial(0));
        }
        for (item_id, (instance_id, needed)) in &selected {
            let item = self.items.iter().find(|item| item.item_id == *item_id);
            if item.is_some_and(|item| item.user_item_id != *instance_id) {
                return Err(EquipmentError::WrongItemInstance { item_id: *item_id });
            }
            let available = item.map_or(0, |item| item.amount);
            if available < *needed {
                return Err(EquipmentError::InsufficientMaterial {
                    item_id: *item_id,
                    needed: *needed,
                    available,
                });
            }
        }

        // Mirrors Data_Equip.ComputeLevelUp, including its gold basis: current stored
        // EXP is included unless the selected materials reach the quality level cap.
        let mut level = equip.level;
        let mut exp = equip.exp.saturating_add(added_exp);
        let mut charged_exp = 0i32;
        while level < max_level {
            let threshold = tables
                .equipment_levels
                .get(level)
                .map(|config| config.exp)
                .filter(|threshold| *threshold > 0)
                .ok_or(EquipmentError::MissingLevelConfig {
                    equip_id: equip.equip_id,
                })?;
            if exp < threshold {
                charged_exp = charged_exp.saturating_add(exp);
                break;
            }
            exp -= threshold;
            charged_exp = charged_exp.saturating_add(threshold);
            level += 1;
        }
        if level == max_level {
            exp = 0;
        }
        let gold_cost =
            (charged_exp as f32 * tables.cultivation_constants.equipment_exp_gold_rate) as i32;
        let coin_item_id = tables.cultivation_constants.coin_item_id;
        let gold_available = self
            .items
            .iter()
            .find(|item| item.item_id == coin_item_id)
            .map_or(0, |item| item.amount);
        if gold_available < gold_cost {
            return Err(EquipmentError::InsufficientGold {
                needed: gold_cost,
                available: gold_available,
            });
        }

        let mut changed_items = Vec::with_capacity(selected.len() + usize::from(gold_cost > 0));
        for (item_id, (_, amount)) in selected {
            let item = self
                .items
                .iter_mut()
                .find(|item| item.item_id == item_id)
                .unwrap();
            item.amount -= amount;
            changed_items.push(*item);
        }
        if gold_cost > 0 {
            let gold = self
                .items
                .iter_mut()
                .find(|item| item.item_id == coin_item_id)
                .unwrap();
            gold.amount -= gold_cost;
            changed_items.push(*gold);
        }
        self.equips[equip_index].level = level;
        self.equips[equip_index].exp = exp;
        append_unlocked_words(
            &mut self.equips[equip_index],
            old_level,
            level,
            self.uid,
            tables,
        );

        // keep cap-overflow refunds empty until a LevelUp capture proves
        // how the authoritative server splits leftover EXP into return materials.
        Ok((self.equips[equip_index].clone(), changed_items, Vec::new()))
    }

    pub fn refresh_equipment_word(
        &mut self,
        equip_id: i64,
        index: i32,
        tables: &GameTables,
    ) -> Result<(i32, Vec<DcNetDataItem>), EquipmentError> {
        let equip_index = self
            .equips
            .iter()
            .position(|equip| equip.user_equip_id == equip_id)
            .ok_or(EquipmentError::UnknownEquipment(equip_id))?;
        let equip = &self.equips[equip_index];
        let word_index =
            usize::try_from(index).map_err(|_| EquipmentError::InvalidWordSlot(index))?;
        // EquipWordReplace unlocks every random-word cell already present in
        // the payload, so existence is the authoritative slot check.
        if word_index >= equip.random_words_id.len() {
            return Err(EquipmentError::InvalidWordSlot(index));
        }
        if equip.minnum >= tables.cultivation_constants.extra_equipment_word_limit {
            return Err(EquipmentError::InvalidWordSlot(index));
        }
        let para = tables.equipment_parameters.get(equip.quality).ok_or(
            EquipmentError::MissingLevelConfig {
                equip_id: equip.equip_id,
            },
        )?;
        for cost in &para.extra_word_costs {
            let available = self
                .items
                .iter()
                .find(|item| item.item_id == cost.key)
                .map_or(0, |item| item.amount);
            if available < cost.value {
                return Err(EquipmentError::InsufficientWordCost(cost.key));
            }
        }

        let occupied_effects: Vec<i32> = equip
            .deputy_words_id
            .iter()
            .chain(
                equip
                    .random_words_id
                    .iter()
                    .enumerate()
                    .filter(|(slot, _)| *slot != word_index)
                    .map(|(_, word)| word),
            )
            .filter_map(|word| tables.extra_equipment_words.get(*word))
            .map(|word| word.effect.key)
            .collect();
        let candidates: Vec<i32> = tables
            .extra_equipment_words_by_slot
            .get(equip.pos)
            .into_iter()
            .flatten()
            .filter(|word| {
                occupied_effects
                    .iter()
                    .filter(|effect| **effect == word.effect.key)
                    .count()
                    < word.duplicate_limit.max(0) as usize
            })
            .map(|word| word.id)
            .collect();
        if candidates.is_empty() {
            return Err(EquipmentError::InvalidWord(0));
        }
        // `word_wheel` is the one-based UI position (EquipWordReplace.Set), not a
        // probability. Keep a stable uniform choice until server evidence proves
        // the actual reroll distribution.
        let seed = (self.uid as u64)
            ^ (equip_id as u64).rotate_left(17)
            ^ (index as u64).rotate_left(33)
            ^ (equip.minnum as u64).rotate_left(49);
        let word_id = candidates[mix64(seed) as usize % candidates.len()];

        let mut remains = Vec::with_capacity(para.extra_word_costs.len());
        for cost in &para.extra_word_costs {
            let item = self
                .items
                .iter_mut()
                .find(|item| item.item_id == cost.key)
                .unwrap();
            item.amount -= cost.value;
            remains.push(*item);
        }
        let equip = &mut self.equips[equip_index];
        equip.tmp_word = word_id;
        equip.tmp_word_idx = index;
        equip.minnum = equip.minnum.saturating_add(1);
        Ok((word_id, remains))
    }

    pub fn replace_equipment_word(
        &mut self,
        equip_id: i64,
        index: i32,
        requested_word: i32,
        replace: bool,
        tables: &GameTables,
    ) -> Result<DcNetDataEquip, EquipmentError> {
        let equip = self
            .equips
            .iter_mut()
            .find(|equip| equip.user_equip_id == equip_id)
            .ok_or(EquipmentError::UnknownEquipment(equip_id))?;
        let word_index =
            usize::try_from(index).map_err(|_| EquipmentError::InvalidWordSlot(index))?;
        if word_index >= equip.random_words_id.len() {
            return Err(EquipmentError::InvalidWordSlot(index));
        }
        if replace {
            let word_id = if requested_word == 0 {
                if equip.tmp_word == 0 || equip.tmp_word_idx != index {
                    return Err(EquipmentError::NoPendingWord(index));
                }
                equip.tmp_word
            } else {
                let valid = equip.minnum >= tables.cultivation_constants.extra_equipment_word_limit
                    && tables
                        .extra_equipment_words
                        .get(requested_word)
                        .is_some_and(|word| word.slot == equip.pos);
                if !valid {
                    return Err(EquipmentError::InvalidWord(requested_word));
                }
                requested_word
            };
            equip.random_words_id[word_index] = word_id;
            equip.minnum = 0;
        } else if equip.tmp_word == 0 || equip.tmp_word_idx != index {
            return Err(EquipmentError::NoPendingWord(index));
        }
        equip.tmp_word = 0;
        equip.tmp_word_idx = -1;
        Ok(equip.clone())
    }

    pub(super) fn has_role(&self, role_id: i32) -> bool {
        self.roles.iter().any(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|role| role.game_role_id == role_id)
        })
    }
}

pub(super) fn initialize_equipment_words(
    equip: &mut DcNetDataEquip,
    uid: i64,
    tables: &GameTables,
) {
    let Some(config) = tables.equipment.get(equip.equip_id) else {
        return;
    };
    if let Some(main_words) = tables.main_equipment_words_by_group.get(config.group_id) {
        let choices: Vec<_> = main_words.collect();
        if !choices.is_empty() {
            let seed = (uid as u64) ^ (equip.user_equip_id as u64).rotate_left(29);
            equip.main_words_id = choices[mix64(seed) as usize % choices.len()].id;
        }
    }
    let count = tables
        .equipment_parameters
        .get(equip.quality)
        .map_or(0, |para| para.max_word_rolls().max(0));
    for roll in 0..count {
        if let Some(word) = roll_extra_word(
            equip,
            (uid as u64)
                ^ (equip.user_equip_id as u64).rotate_left(17)
                ^ (roll as u64).rotate_left(41),
            tables,
        ) {
            equip.deputy_words_id.push(word);
        }
    }
}

pub(super) fn normalize_equipment_words(
    equip: &mut DcNetDataEquip,
    planned_words: &mut Vec<i32>,
    tables: &GameTables,
) {
    let Some(parameters) = tables.equipment_parameters.get(equip.quality) else {
        return;
    };
    let max_rolls = parameters.max_word_rolls() as usize;
    if !planned_words.is_empty() {
        equip.deputy_words_id.append(planned_words);
    }
    equip.deputy_words_id.truncate(max_rolls);
    while equip.deputy_words_id.len() < max_rolls {
        let roll = equip.deputy_words_id.len() as u64;
        let seed = (equip.user_equip_id as u64)
            ^ (equip.equip_id as u64).rotate_left(19)
            ^ roll.rotate_left(41);
        let Some(word) = roll_extra_word(equip, seed, tables) else {
            break;
        };
        equip.deputy_words_id.push(word);
    }
    equip
        .random_words_id
        .truncate(parameters.breakpoint_word_rolls.max(0) as usize);
    if equip.tmp_word != 0
        && usize::try_from(equip.tmp_word_idx)
            .ok()
            .is_none_or(|index| index >= equip.random_words_id.len())
    {
        equip.tmp_word = 0;
        equip.tmp_word_idx = -1;
    }
}

fn append_unlocked_words(
    equip: &mut DcNetDataEquip,
    old_level: i32,
    new_level: i32,
    uid: i64,
    tables: &GameTables,
) {
    let crossed_breakpoints = tables
        .equipment_levels
        .rows
        .iter()
        .filter(|level| level.breakpoint > 0 && level.level > old_level && level.level <= new_level)
        .count();
    let Some(parameters) = tables.equipment_parameters.get(equip.quality) else {
        return;
    };
    let remaining_random_slots = (parameters.breakpoint_word_rolls.max(0) as usize)
        .saturating_sub(equip.random_words_id.len());
    for _ in 0..crossed_breakpoints.min(remaining_random_slots) {
        let salt = equip.random_words_id.len() as u64;
        let seed = (uid as u64)
            ^ (equip.user_equip_id as u64).rotate_left(17)
            ^ salt.rotate_left(23)
            ^ (new_level as u64).rotate_left(47);
        let replacement = roll_existing_word(equip, seed);
        if let Some(word) = replacement {
            equip.random_words_id.push(word);
        } else {
            break;
        }
    }
}

fn roll_extra_word(equip: &DcNetDataEquip, seed: u64, tables: &GameTables) -> Option<i32> {
    roll_word(equip, seed, tables, true)
}

fn roll_existing_word(equip: &DcNetDataEquip, seed: u64) -> Option<i32> {
    let words: Vec<i32> = equip
        .deputy_words_id
        .iter()
        .chain(&equip.random_words_id)
        .copied()
        .collect();
    (!words.is_empty()).then(|| words[mix64(seed) as usize % words.len()])
}

fn roll_word(
    equip: &DcNetDataEquip,
    seed: u64,
    tables: &GameTables,
    allow_duplicates: bool,
) -> Option<i32> {
    let effects: Vec<i32> = equip
        .deputy_words_id
        .iter()
        .chain(&equip.random_words_id)
        .filter_map(|word| tables.extra_equipment_words.get(*word))
        .map(|word| word.effect.key)
        .collect();
    let candidates: Vec<_> = tables
        .extra_equipment_words_by_slot
        .get(equip.pos)?
        .filter(|word| {
            let count = effects
                .iter()
                .filter(|effect| **effect == word.effect.key)
                .count();
            if allow_duplicates {
                count < word.duplicate_limit.max(0) as usize
            } else {
                count == 0
            }
        })
        .collect();
    (!candidates.is_empty()).then(|| candidates[mix64(seed) as usize % candidates.len()].id)
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
#[cfg(test)]
mod tests;
