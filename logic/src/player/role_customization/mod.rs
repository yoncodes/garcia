use protocol::pbcommon::{
    DcNetDataEquip, DcNetDataFormation, DcNetDataPartner, DcNetDataRoleBasicInfo,
    DcNetDataRolesChangeAppear, DcNetDataRolesRolesDetail, DcNetDataSkillStone,
};

use super::*;

pub struct MainCharacterSwitchOutcome {
    pub previous: DcNetDataRolesRolesDetail,
    pub current: DcNetDataRolesRolesDetail,
    pub dropped_equips: Vec<DcNetDataEquip>,
    pub dropped_partner: Option<DcNetDataPartner>,
    pub changed_formations: Vec<DcNetDataFormation>,
    pub dropped_skillstones: Vec<DcNetDataSkillStone>,
}

impl Player {
    pub fn change_role_appearance(
        &mut self,
        change: DcNetDataRolesChangeAppear,
    ) -> Result<DcNetDataRolesChangeAppear, RoleMutationError> {
        if !matches!(change.key, 1 | 2) {
            return Err(RoleMutationError::InvalidAppearKey(change.key));
        }
        self.role_info_mut(change.game_role_id)?.appear_skill_key = change.key;
        Ok(change)
    }

    pub fn change_role_element(
        &mut self,
        role_id: i32,
        element: i32,
        for_call: bool,
    ) -> Result<DcNetDataRoleBasicInfo, RoleMutationError> {
        if !(1..=5).contains(&element) {
            return Err(RoleMutationError::InvalidElement(element));
        }
        let role = self.role_info_mut(role_id)?;
        if for_call {
            role.element4call = element;
        } else {
            role.element = element;
        }
        Ok(role.clone())
    }

    pub fn change_banner_girl(&mut self, role_id: i32) -> Result<i32, RoleMutationError> {
        self.role_info_mut(role_id)?;
        self.banner_girl = role_id;
        Ok(role_id)
    }

    pub fn role_profiles(&self) -> Vec<i32> {
        let mut profiles = self
            .roles
            .iter()
            .filter_map(|role| role.role_basic_info.as_ref().map(|info| info.game_role_id))
            .collect::<Vec<_>>();
        profiles.sort_unstable();
        profiles.dedup();
        profiles
    }

    pub fn switch_main_character(
        &mut self,
        target_id: i32,
        tables: &GameTables,
    ) -> Result<MainCharacterSwitchOutcome, RoleMutationError> {
        if tables
            .maids
            .get(target_id)
            .is_none_or(|maid| maid.maid_group == 0)
        {
            return Err(RoleMutationError::NotMainCharacter(target_id));
        }
        let previous_index = self
            .roles
            .iter()
            .position(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|info| info.cur_mc)
            })
            .ok_or(RoleMutationError::MissingMainCharacter)?;
        let previous_id = self.roles[previous_index]
            .role_basic_info
            .as_ref()
            .unwrap()
            .game_role_id;
        if previous_id == target_id {
            return Err(RoleMutationError::AlreadyMainCharacter(target_id));
        }

        let target_index = self
            .roles
            .iter()
            .position(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|info| info.game_role_id == target_id)
            })
            .or_else(|| {
                self.roles.push(gacha_role(self.uid, target_id, tables)?);
                Some(self.roles.len() - 1)
            })
            .ok_or(RoleMutationError::UnknownRole(target_id))?;

        let dropped_equips = self
            .equips
            .iter_mut()
            .filter(|equip| equip.equiped_role == previous_id)
            .map(|equip| {
                equip.equiped_role = 0;
                equip.clone()
            })
            .collect();
        let dropped_partner = self.roles[previous_index]
            .role_basic_info
            .as_ref()
            .and_then(|info| {
                self.partners
                    .iter()
                    .find(|partner| partner.id == info.user_partner_id)
                    .copied()
            });
        let changed_formations = self
            .formations
            .iter_mut()
            .filter_map(|formation| {
                let before = formation.poss.len();
                formation
                    .poss
                    .retain(|position| position.game_role_id != previous_id);
                (formation.poss.len() != before).then(|| formation.clone())
            })
            .collect();
        let dropped_skillstones = self
            .skillstones
            .iter_mut()
            .filter(|stone| stone.equiped_role == previous_id)
            .map(|stone| {
                stone.equiped_role = 0;
                stone.pos = 0;
                *stone
            })
            .collect();
        self.roles[previous_index]
            .role_basic_info
            .as_mut()
            .unwrap()
            .user_partner_id = 0;
        self.roles[previous_index]
            .role_basic_info
            .as_mut()
            .unwrap()
            .cur_mc = false;
        self.roles[target_index]
            .role_basic_info
            .as_mut()
            .unwrap()
            .cur_mc = true;
        self.sync_role_skillstones();

        Ok(MainCharacterSwitchOutcome {
            previous: self.roles[previous_index].clone(),
            current: self.roles[target_index].clone(),
            dropped_equips,
            dropped_partner,
            changed_formations,
            dropped_skillstones,
        })
    }

    fn role_info_mut(
        &mut self,
        role_id: i32,
    ) -> Result<&mut DcNetDataRoleBasicInfo, RoleMutationError> {
        self.roles
            .iter_mut()
            .filter_map(|role| role.role_basic_info.as_mut())
            .find(|role| role.game_role_id == role_id)
            .ok_or(RoleMutationError::UnknownRole(role_id))
    }
}
#[cfg(test)]
mod tests;
