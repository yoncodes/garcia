use super::*;

impl Player {
    pub fn grant_skillstone(
        &mut self,
        stone_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataSkillStone, SkillStoneError> {
        let config = tables
            .skill_stones
            .get(stone_id)
            .ok_or(SkillStoneError::MissingConfig(stone_id))?;
        let base = make_entity_id(self.uid, stone_id);
        let user_stone_id = (0..=self.skillstones.len())
            .filter_map(|offset| i64::try_from(offset).ok())
            .filter_map(|offset| base.checked_add(offset))
            .find(|candidate| {
                !self
                    .skillstones
                    .iter()
                    .any(|stone| stone.user_stone_id == *candidate)
            })
            .ok_or(SkillStoneError::InstanceIdExhausted)?;
        let stone = DcNetDataSkillStone {
            user_stone_id,
            stone_id,
            quality: config.quality,
            ..Default::default()
        };
        self.skillstones.push(stone);
        Ok(stone)
    }

    pub fn set_skillstone(
        &mut self,
        role_id: i32,
        user_stone_id: i64,
        position: i32,
        tables: &GameTables,
    ) -> Result<(DcNetDataSkillStone, Vec<DcNetDataSkillStone>), SkillStoneError> {
        if !self.has_role(role_id) {
            return Err(SkillStoneError::UnknownRole(role_id));
        }
        let stone_index = self
            .skillstones
            .iter()
            .position(|stone| stone.user_stone_id == user_stone_id)
            .ok_or(SkillStoneError::UnknownStone(user_stone_id))?;
        self.validate_skillstone_role(stone_index, role_id, tables)?;

        let dropped = self
            .skillstones
            .iter_mut()
            .enumerate()
            .filter(|(index, stone)| {
                *index != stone_index && stone.equiped_role == role_id && stone.pos == position
            })
            .map(|(_, stone)| {
                stone.equiped_role = 0;
                stone.pos = 0;
                *stone
            })
            .collect();
        self.skillstones[stone_index].equiped_role = role_id;
        self.skillstones[stone_index].pos = position;
        let stone = self.skillstones[stone_index];
        self.sync_role_skillstones();
        Ok((stone, dropped))
    }

    pub fn unset_skillstone(
        &mut self,
        role_id: i32,
        position: i32,
    ) -> Result<Vec<DcNetDataSkillStone>, SkillStoneError> {
        if !self.has_role(role_id) {
            return Err(SkillStoneError::UnknownRole(role_id));
        }
        let dropped = self
            .skillstones
            .iter_mut()
            .filter(|stone| stone.equiped_role == role_id && stone.pos == position)
            .map(|stone| {
                stone.equiped_role = 0;
                stone.pos = 0;
                *stone
            })
            .collect();
        self.sync_role_skillstones();
        Ok(dropped)
    }

    pub fn lock_skillstone(
        &mut self,
        user_stone_id: i64,
        locked: bool,
    ) -> Result<DcNetDataSkillStone, SkillStoneError> {
        let stone = self
            .skillstones
            .iter_mut()
            .find(|stone| stone.user_stone_id == user_stone_id)
            .ok_or(SkillStoneError::UnknownStone(user_stone_id))?;
        stone.locked = locked;
        let stone = *stone;
        self.sync_role_skillstones();
        Ok(stone)
    }

    pub fn swap_skillstones(
        &mut self,
        first_id: i64,
        second_id: i64,
        tables: &GameTables,
    ) -> Result<(DcNetDataSkillStone, DcNetDataSkillStone), SkillStoneError> {
        let first = self
            .skillstones
            .iter()
            .position(|stone| stone.user_stone_id == first_id)
            .ok_or(SkillStoneError::UnknownStone(first_id))?;
        let second = self
            .skillstones
            .iter()
            .position(|stone| stone.user_stone_id == second_id)
            .ok_or(SkillStoneError::UnknownStone(second_id))?;
        if first == second {
            return Ok((self.skillstones[first], self.skillstones[second]));
        }
        let first_slot = (
            self.skillstones[first].equiped_role,
            self.skillstones[first].pos,
        );
        let second_slot = (
            self.skillstones[second].equiped_role,
            self.skillstones[second].pos,
        );
        if second_slot.0 != 0 {
            self.validate_skillstone_role(first, second_slot.0, tables)?;
        }
        if first_slot.0 != 0 {
            self.validate_skillstone_role(second, first_slot.0, tables)?;
        }
        (
            self.skillstones[first].equiped_role,
            self.skillstones[first].pos,
        ) = second_slot;
        (
            self.skillstones[second].equiped_role,
            self.skillstones[second].pos,
        ) = first_slot;
        let result = (self.skillstones[first], self.skillstones[second]);
        self.sync_role_skillstones();
        Ok(result)
    }

    pub(super) fn sync_role_skillstones(&mut self) {
        for role in &mut self.roles {
            let role_id = role
                .role_basic_info
                .as_ref()
                .map_or(0, |info| info.game_role_id);
            role.skillstones = self
                .skillstones
                .iter()
                .filter(|stone| stone.equiped_role == role_id)
                .copied()
                .collect();
        }
    }

    fn validate_skillstone_role(
        &self,
        stone_index: usize,
        role_id: i32,
        tables: &GameTables,
    ) -> Result<(), SkillStoneError> {
        let stone = &self.skillstones[stone_index];
        let config = tables
            .skill_stones
            .get(stone.stone_id)
            .ok_or(SkillStoneError::MissingConfig(stone.stone_id))?;
        if config.under_maid != 0 && config.under_maid != role_id {
            return Err(SkillStoneError::WrongRole {
                stone_id: stone.user_stone_id,
                expected: config.under_maid,
                actual: role_id,
            });
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
