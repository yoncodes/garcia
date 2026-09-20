use std::collections::HashSet;

use super::*;

impl Player {
    pub fn save_role_attrs(
        &mut self,
        role_attrs: Vec<DcNetDataRoleAttrInfo>,
    ) -> Result<(), RoleAttrError> {
        let mut seen = HashSet::with_capacity(role_attrs.len());
        for attr in &role_attrs {
            if !seen.insert(attr.role_id) {
                return Err(RoleAttrError::DuplicateRole(attr.role_id));
            }
            if attr.mp < 0 || attr.ep < 0 || attr.hp < 0 {
                return Err(RoleAttrError::NegativeAttribute(attr.role_id));
            }
            if !self.roles.iter().any(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|role| role.game_role_id == attr.role_id)
            }) {
                return Err(RoleAttrError::RoleNotOwned(attr.role_id));
            }
        }
        self.role_attrs = role_attrs;
        self.role_attrs.sort_by_key(|attr| attr.role_id);
        Ok(())
    }
}
#[cfg(test)]
mod tests;
