use super::*;

impl Player {
    pub fn team_equip_details(&self) -> Vec<DcNetDataTeamEquipDetail> {
        self.team_equips
            .iter()
            .map(|equip| DcNetDataTeamEquipDetail {
                info: Some(*equip),
                core_info: self
                    .team_cores
                    .iter()
                    .filter(|core| core.equiped_id == equip.id)
                    .map(|core| (core.pos, core.id))
                    .collect(),
            })
            .collect()
    }

    pub fn grant_team_equip(
        &mut self,
        config_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataTeamEquip, TeamEquipError> {
        let config = tables
            .team_equipment
            .get(config_id)
            .ok_or(TeamEquipError::MissingConfig(config_id))?;
        let param = tables
            .team_equipment_parameters
            .get(config.quality)
            .ok_or(TeamEquipError::MissingConfig(config_id))?;
        let base = make_entity_id(self.uid, config_id);
        let instance_id = (0..=self.team_equips.len())
            .filter_map(|offset| i64::try_from(offset).ok())
            .filter_map(|offset| base.checked_add(offset))
            .find(|candidate| !self.team_equips.iter().any(|equip| equip.id == *candidate))
            .ok_or(TeamEquipError::InstanceIdExhausted)?;
        let equip = DcNetDataTeamEquip {
            id: instance_id,
            te_id: config_id,
            main_words_id: config.main_word_id,
            pos_num: param.core_slots,
            ..Default::default()
        };
        self.team_equips.push(equip);
        Ok(equip)
    }

    pub fn equip_team_equip(
        &mut self,
        instance_id: i64,
        formation_id: i32,
        tables: &GameTables,
    ) -> Result<(DcNetDataTeamEquip, i32), TeamEquipError> {
        if !self
            .formations
            .iter()
            .any(|formation| formation.formation_id == formation_id)
        {
            return Err(TeamEquipError::UnknownFormation(formation_id));
        }
        let equip_index = self
            .team_equips
            .iter()
            .position(|equip| equip.id == instance_id)
            .ok_or(TeamEquipError::UnknownEquip(instance_id))?;
        let equip_type = tables
            .team_equipment
            .get(self.team_equips[equip_index].te_id)
            .ok_or(TeamEquipError::MissingConfig(
                self.team_equips[equip_index].te_id,
            ))?
            .equipment_type;
        let drop_fid = self.team_equips[equip_index].equiped_fid;

        for (index, equip) in self.team_equips.iter_mut().enumerate() {
            if index == equip_index || equip.equiped_fid != formation_id {
                continue;
            }
            let same_type = tables
                .team_equipment
                .get(equip.te_id)
                .is_some_and(|config| config.equipment_type == equip_type);
            if same_type {
                equip.equiped_fid = 0;
            }
        }
        self.team_equips[equip_index].equiped_fid = formation_id;
        Ok((self.team_equips[equip_index], drop_fid))
    }

    pub fn unequip_team_equip(&mut self, instance_id: i64) -> Result<i32, TeamEquipError> {
        let equip = self
            .team_equips
            .iter_mut()
            .find(|equip| equip.id == instance_id)
            .ok_or(TeamEquipError::UnknownEquip(instance_id))?;
        let drop_fid = equip.equiped_fid;
        equip.equiped_fid = 0;
        Ok(drop_fid)
    }

    pub fn lock_team_equip(
        &mut self,
        instance_id: i64,
        state: bool,
    ) -> Result<DcNetDataTeamEquip, TeamEquipError> {
        let equip = self
            .team_equips
            .iter_mut()
            .find(|equip| equip.id == instance_id)
            .ok_or(TeamEquipError::UnknownEquip(instance_id))?;
        equip.locked = state;
        Ok(*equip)
    }

    pub fn level_up_team_equip(
        &mut self,
        instance_id: i64,
        materials: &[DcNetDataItem],
        material_equips: &[i64],
        tables: &GameTables,
    ) -> Result<(DcNetDataTeamEquip, Vec<DcNetDataItem>), TeamEquipError> {
        let equip_index = self
            .team_equips
            .iter()
            .position(|equip| equip.id == instance_id)
            .ok_or(TeamEquipError::UnknownEquip(instance_id))?;
        let target_config = tables
            .team_equipment
            .get(self.team_equips[equip_index].te_id)
            .ok_or(TeamEquipError::MissingConfig(
                self.team_equips[equip_index].te_id,
            ))?;
        let target_param = tables
            .team_equipment_parameters
            .get(target_config.quality)
            .ok_or(TeamEquipError::MissingConfig(target_config.id))?;
        if self.team_equips[equip_index].lv >= target_param.max_level {
            return Err(TeamEquipError::LevelCap(instance_id));
        }

        let mut selected_items = BTreeMap::<i32, (i64, i32)>::new();
        let mut added_exp = 0i32;
        for material in materials {
            if material.amount <= 0 {
                return Err(TeamEquipError::InvalidMaterial(material.item_id));
            }
            let effect = tables
                .items
                .get(material.item_id)
                .filter(|item| item.sub_type == 24)
                .and_then(|item| item.effect.as_ref())
                .filter(|effect| effect.key == 100_100_010 && effect.value > 0)
                .ok_or(TeamEquipError::InvalidMaterial(material.item_id))?;
            let selected = selected_items
                .entry(material.item_id)
                .or_insert((material.user_item_id, 0));
            if selected.0 != material.user_item_id {
                return Err(TeamEquipError::WrongItemInstance {
                    item_id: material.item_id,
                });
            }
            selected.1 = selected.1.saturating_add(material.amount);
            added_exp = added_exp.saturating_add(effect.value.saturating_mul(material.amount));
        }
        for (item_id, (instance_id, needed)) in &selected_items {
            let item = self.items.iter().find(|item| item.item_id == *item_id);
            if item.is_some_and(|item| item.user_item_id != *instance_id) {
                return Err(TeamEquipError::WrongItemInstance { item_id: *item_id });
            }
            let available = item.map_or(0, |item| item.amount);
            if available < *needed {
                return Err(TeamEquipError::InsufficientMaterial {
                    item_id: *item_id,
                    needed: *needed,
                    available,
                });
            }
        }

        let mut selected_equips = Vec::with_capacity(material_equips.len());
        for material_id in material_equips {
            if *material_id == instance_id || selected_equips.contains(material_id) {
                return Err(TeamEquipError::DuplicateMaterial(*material_id));
            }
            let material = self
                .team_equips
                .iter()
                .find(|equip| equip.id == *material_id)
                .ok_or(TeamEquipError::UnknownEquip(*material_id))?;
            if material.locked
                || material.equiped_fid != 0
                || self
                    .team_cores
                    .iter()
                    .any(|core| core.equiped_id == *material_id)
            {
                return Err(TeamEquipError::MaterialUnavailable(*material_id));
            }
            let config = tables
                .team_equipment
                .get(material.te_id)
                .ok_or(TeamEquipError::MissingConfig(material.te_id))?;
            let param = tables
                .team_equipment_parameters
                .get(config.quality)
                .ok_or(TeamEquipError::MissingConfig(config.id))?;
            let invested_exp = (0..material.lv).try_fold(0i32, |total, level| {
                tables
                    .team_equipment_levels
                    .get(level)
                    .filter(|row| row.exp > 0)
                    .map(|row| total.saturating_add(row.exp))
                    .ok_or(TeamEquipError::MissingConfig(level))
            })?;
            added_exp = added_exp
                .saturating_add(param.exp)
                .saturating_add(invested_exp)
                .saturating_add(material.exp);
            selected_equips.push(*material_id);
        }
        if selected_items.is_empty() && selected_equips.is_empty() {
            return Err(TeamEquipError::InvalidMaterial(0));
        }

        let mut level = self.team_equips[equip_index].lv;
        let mut exp = self.team_equips[equip_index].exp.saturating_add(added_exp);
        while level < target_param.max_level {
            let threshold = tables
                .team_equipment_levels
                .get(level)
                .map(|row| row.exp)
                .filter(|threshold| *threshold > 0)
                .ok_or(TeamEquipError::MissingConfig(level))?;
            if exp < threshold {
                break;
            }
            exp -= threshold;
            level += 1;
        }
        if level == target_param.max_level {
            exp = 0;
        }

        let mut remains = Vec::with_capacity(selected_items.len());
        for (item_id, (_, amount)) in selected_items {
            let item = self
                .items
                .iter_mut()
                .find(|item| item.item_id == item_id)
                .expect("team-equipment materials were validated");
            item.amount -= amount;
            remains.push(*item);
        }
        self.team_equips[equip_index].lv = level;
        self.team_equips[equip_index].exp = exp;
        let upgraded = self.team_equips[equip_index];
        self.team_equips
            .retain(|equip| !selected_equips.contains(&equip.id));
        Ok((upgraded, remains))
    }

    pub fn set_team_equip_core(
        &mut self,
        equip_id: i64,
        core_id: i64,
        position: i32,
        tables: &GameTables,
    ) -> Result<(DcNetDataTeamCore, Option<DcNetDataTeamCore>), TeamEquipError> {
        let equip = self
            .team_equips
            .iter()
            .find(|equip| equip.id == equip_id)
            .ok_or(TeamEquipError::UnknownEquip(equip_id))?;
        let config = tables
            .team_equipment
            .get(equip.te_id)
            .ok_or(TeamEquipError::MissingConfig(equip.te_id))?;
        let max_position = tables
            .team_equipment_parameters
            .get(config.quality)
            .ok_or(TeamEquipError::MissingConfig(equip.te_id))?
            .core_slots;
        if !(1..=max_position).contains(&position) {
            return Err(TeamEquipError::InvalidCorePosition { equip_id, position });
        }
        let core_index = self
            .team_cores
            .iter()
            .position(|core| core.id == core_id)
            .ok_or(TeamEquipError::UnknownCore(core_id))?;
        let dropped_index = self.team_cores.iter().position(|core| {
            core.id != core_id && core.equiped_id == equip_id && core.pos == position
        });
        let dropped = dropped_index.map(|index| {
            self.team_cores[index].equiped_id = 0;
            self.team_cores[index].pos = 0;
            self.team_cores[index]
        });
        self.team_cores[core_index].equiped_id = equip_id;
        self.team_cores[core_index].pos = position;
        Ok((self.team_cores[core_index], dropped))
    }

    pub fn unset_team_equip_core(
        &mut self,
        equip_id: i64,
        position: i32,
    ) -> Result<Option<DcNetDataTeamCore>, TeamEquipError> {
        if !self.team_equips.iter().any(|equip| equip.id == equip_id) {
            return Err(TeamEquipError::UnknownEquip(equip_id));
        }
        Ok(self
            .team_cores
            .iter_mut()
            .find(|core| core.equiped_id == equip_id && core.pos == position)
            .map(|core| {
                core.equiped_id = 0;
                core.pos = 0;
                *core
            }))
    }
}
#[cfg(test)]
mod tests;
