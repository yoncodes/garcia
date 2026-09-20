use super::*;

impl Player {
    pub fn compose_team_cores(
        &mut self,
        synthesis_id: i32,
        amount: i32,
        tables: &GameTables,
    ) -> Result<(Vec<DcNetDataTeamCore>, Vec<DcNetDataItem>), TeamCoreError> {
        if amount <= 0 {
            return Err(TeamCoreError::InvalidAmount);
        }
        let synthesis = tables
            .team_core_recipes
            .get(synthesis_id)
            .ok_or(TeamCoreError::UnknownSynthesis(synthesis_id))?;
        let total_weight = synthesis
            .target
            .iter()
            .filter(|entry| entry.value > 0 && tables.team_cores.get(entry.key).is_some())
            .map(|entry| entry.value)
            .sum::<i32>();
        if total_weight <= 0 {
            return Err(TeamCoreError::InvalidTargets(synthesis_id));
        }

        let costs = synthesis
            .cost_item
            .iter()
            .map(|cost| {
                cost.value
                    .checked_mul(amount)
                    .filter(|needed| *needed > 0)
                    .map(|needed| (cost.key, needed))
                    .ok_or(TeamCoreError::InvalidAmount)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let remains = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            TeamCoreError::InsufficientCost {
                item_id,
                needed,
                available,
            }
        })?;

        let mut created = Vec::with_capacity(amount as usize);
        for offset in 0..amount {
            let roll = deterministic_roll(
                self.uid as u64
                    ^ (self.team_cores.len() as u64).rotate_left(17)
                    ^ (synthesis_id as u64).rotate_left(33)
                    ^ offset as u64,
                total_weight as u64,
            ) as i32;
            let mut cursor = 0;
            let core_id = synthesis
                .target
                .iter()
                .filter(|entry| entry.value > 0 && tables.team_cores.get(entry.key).is_some())
                .find_map(|entry| {
                    cursor += entry.value;
                    (roll < cursor).then_some(entry.key)
                })
                .ok_or(TeamCoreError::InvalidTargets(synthesis_id))?;
            created.push(self.grant_team_core(core_id, tables)?);
        }
        Ok((created, remains))
    }

    pub fn lock_team_core(
        &mut self,
        instance_id: i64,
        state: bool,
    ) -> Result<DcNetDataTeamCore, TeamCoreError> {
        let core = self
            .team_cores
            .iter_mut()
            .find(|core| core.id == instance_id)
            .ok_or(TeamCoreError::UnknownCore(instance_id))?;
        core.locked = state;
        Ok(*core)
    }

    pub fn grant_team_core(
        &mut self,
        core_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataTeamCore, TeamCoreError> {
        if tables.team_cores.get(core_id).is_none() {
            return Err(TeamCoreError::MissingConfig(core_id));
        }
        let base = make_entity_id(self.uid, core_id);
        let instance_id = (0..=self.team_cores.len())
            .filter_map(|offset| i64::try_from(offset).ok())
            .filter_map(|offset| base.checked_add(offset))
            .find(|candidate| !self.team_cores.iter().any(|core| core.id == *candidate))
            .ok_or(TeamCoreError::InvalidTargets(core_id))?;
        let core = DcNetDataTeamCore {
            id: instance_id,
            core_id,
            ..Default::default()
        };
        self.team_cores.push(core);
        Ok(core)
    }
}

fn deterministic_roll(mut state: u64, upper: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state % upper
}
#[cfg(test)]
mod tests;
