use super::*;

impl Player {
    pub fn grant_all_profile_frames(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> DcNetDataTakeRewardRes {
        let missing = tables
            .profile_frames
            .rows
            .iter()
            .map(|profile| profile.id)
            .filter(|id| !self.profile_frames.iter().any(|owned| owned.id == *id))
            .collect::<Vec<_>>();
        let mut rewards = DcNetDataTakeRewardRes::default();
        for id in missing {
            self.add_reward(id, 1, tables, now, &mut rewards);
        }
        rewards
    }

    pub fn grant_all_profile_titles(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> DcNetDataTakeRewardRes {
        let missing = tables
            .profile_titles
            .rows
            .iter()
            .map(|profile| profile.id)
            .filter(|id| !self.profile_titles.iter().any(|owned| owned.id == *id))
            .collect::<Vec<_>>();
        let mut rewards = DcNetDataTakeRewardRes::default();
        for id in missing {
            self.add_reward(id, 1, tables, now, &mut rewards);
        }
        rewards
    }

    pub fn select_profile(
        &mut self,
        profile_type: i32,
        id: i32,
    ) -> Result<bool, ProfileSelectionError> {
        let current = match profile_type {
            1 if self.profile_avatars.iter().any(|entry| entry.id == id) => {
                &mut self.profile_avatar
            }
            2 if self.profile_frames.iter().any(|entry| entry.id == id) => &mut self.profile_frame,
            3 if self.profile_titles.iter().any(|entry| entry.id == id) => &mut self.profile_title,
            4 if self.profile_cards.iter().any(|entry| entry.id == id) => &mut self.profile_card,
            _ => {
                return Err(ProfileSelectionError::Locked { profile_type, id });
            }
        };
        if *current == id {
            return Ok(false);
        }
        *current = id;
        Ok(true)
    }

    pub(super) fn unlock_role_avatar(
        &mut self,
        maid_id: i32,
        tables: &GameTables,
    ) -> Option<DcNetDataProfileAvatar> {
        let id = tables.profile_avatar_for_maid(maid_id)?.id;
        if self.profile_avatars.iter().any(|avatar| avatar.id == id) {
            return None;
        }
        let avatar = DcNetDataProfileAvatar { id };
        self.profile_avatars.push(avatar);
        Some(avatar)
    }

    pub fn unlock_maxed_role_namecard(
        &mut self,
        maid_id: i32,
        tables: &GameTables,
    ) -> Option<DcNetDataProfileCard> {
        let is_maxed = self.roles.iter().any(|role| {
            role.role_basic_info.as_ref().is_some_and(|role| {
                role.game_role_id == maid_id
                    && role.maid_qua >= tables.cultivation_constants.max_role_resonance
            })
        });
        if !is_maxed {
            return None;
        }

        let id = tables.profile_card_for_maid(maid_id)?.id;
        if self.profile_cards.iter().any(|card| card.id == id) {
            return None;
        }
        let card = DcNetDataProfileCard { id };
        self.profile_cards.push(card);
        Some(card)
    }

    pub fn rename(
        &mut self,
        name_type: i32,
        name: String,
        gender: i32,
        tables: &GameTables,
    ) -> Result<Vec<DcNetDataItem>, NamingError> {
        if !matches!(name_type, 1 | 2) {
            return Err(NamingError::InvalidType(name_type));
        }
        let length = name.encode_utf16().count();
        if !(1..=12).contains(&length)
            || !name.chars().all(|character| {
                character.is_ascii_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&character)
            })
        {
            return Err(NamingError::InvalidName);
        }

        if name_type == 1 {
            self.choose_gender(gender, tables)?;
        }

        let remains = if name_type == 2 {
            let cost = tables
                .cultivation_constants
                .rename_cost
                .first()
                .ok_or(NamingError::MissingCost)?;
            let costs = [(cost.key, cost.value)].into_iter().collect();
            consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
                NamingError::InsufficientCost {
                    item_id,
                    needed,
                    available,
                }
            })?
        } else {
            Vec::new()
        };
        self.nickname = name;
        Ok(remains)
    }

    pub fn gender(&self, tables: &GameTables) -> i32 {
        let current = self.roles.iter().find_map(|role| {
            role.role_basic_info
                .as_ref()
                .filter(|role| role.cur_mc)
                .map(|role| role.game_role_id)
        });
        current
            .and_then(|id| {
                tables
                    .cultivation_constants
                    .default_main_maid_sex
                    .iter()
                    .position(|candidate| *candidate == id)
            })
            .map_or(0, |gender| gender as i32)
    }

    fn choose_gender(&mut self, gender: i32, tables: &GameTables) -> Result<(), NamingError> {
        let main_characters = &tables.cultivation_constants.default_main_maid_sex;
        let selected = usize::try_from(gender)
            .ok()
            .and_then(|gender| main_characters.get(gender))
            .copied()
            .filter(|id| tables.maids.get(*id).is_some())
            .ok_or(NamingError::InvalidGender(gender))?;

        if !self.roles.iter().any(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|role| role.game_role_id == selected)
        }) {
            self.roles
                .push(gacha_role(self.uid, selected, tables).unwrap());
        }
        for role in self
            .roles
            .iter_mut()
            .filter_map(|role| role.role_basic_info.as_mut())
        {
            role.cur_mc = role.game_role_id == selected;
        }
        for position in self
            .formations
            .iter_mut()
            .flat_map(|formation| &mut formation.poss)
            .filter(|position| main_characters.contains(&position.game_role_id))
        {
            position.game_role_id = selected;
        }

        self.banner_girl = selected;
        if let Some(avatar) = tables.profile_avatar_for_maid(selected) {
            self.profile_avatar = avatar.id;
            if !self
                .profile_avatars
                .iter()
                .any(|profile| profile.id == avatar.id)
            {
                self.profile_avatars
                    .push(DcNetDataProfileAvatar { id: avatar.id });
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
