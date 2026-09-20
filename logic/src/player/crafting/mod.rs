use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftOutcome {
    pub id: i32,
    pub amount: i32,
    pub remains: Vec<DcNetDataItem>,
    pub target: DcNetDataItem,
}

impl Player {
    pub fn synthesize_item(
        &mut self,
        id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<CraftOutcome, CraftError> {
        let formula = tables
            .item_synthesis_recipes
            .get(id)
            .cloned()
            .ok_or(CraftError::UnknownSynthesis(id))?;
        validate_craft(
            id,
            amount,
            formula.target,
            &formula.conditions,
            self,
            tables,
            now,
        )?;

        let mut costs = BTreeMap::new();
        for cost in formula.cost_item {
            let needed = cost
                .value
                .checked_mul(amount)
                .ok_or(CraftError::InvalidAmount(amount))?;
            *costs.entry(cost.key).or_default() += needed;
        }
        finish_craft(self, id, amount, formula.target, costs, tables)
    }

    pub fn convert_item(
        &mut self,
        id: i32,
        amount: i32,
        selected_costs: &[i32],
        tables: &GameTables,
        now: i32,
    ) -> Result<CraftOutcome, CraftError> {
        let formula = tables
            .item_conversions
            .get(id)
            .cloned()
            .ok_or(CraftError::UnknownConversion(id))?;
        validate_craft(
            id,
            amount,
            formula.target,
            &formula.conditions,
            self,
            tables,
            now,
        )?;
        let expected = usize::try_from(formula.cost_type_number).unwrap_or_default();
        if selected_costs.len() != expected {
            return Err(CraftError::WrongMaterialCount {
                expected,
                actual: selected_costs.len(),
            });
        }

        let per_slot = formula
            .cost_item_number
            .checked_mul(amount)
            .ok_or(CraftError::InvalidAmount(amount))?;
        let mut costs = BTreeMap::new();
        for &item_id in selected_costs {
            if !formula.cost_item.contains(&item_id) {
                return Err(CraftError::InvalidMaterial(item_id));
            }
            *costs.entry(item_id).or_default() += per_slot;
        }
        finish_craft(self, id, amount, formula.target, costs, tables)
    }
}

fn validate_craft(
    id: i32,
    amount: i32,
    target: i32,
    limits: &[configs::tables::TablePair<String>],
    player: &Player,
    tables: &GameTables,
    now: i32,
) -> Result<(), CraftError> {
    if amount <= 0 {
        return Err(CraftError::InvalidAmount(amount));
    }
    if tables.items.get(target).is_none() {
        return Err(CraftError::UnknownTarget(target));
    }
    if !limits.iter().all(|limit| {
        player.condition_progress(limit.key, &limit.value, tables, now)
            >= condition_total(limit.key, &limit.value)
    }) {
        return Err(CraftError::Locked(id));
    }
    Ok(())
}

fn finish_craft(
    player: &mut Player,
    id: i32,
    amount: i32,
    target_id: i32,
    costs: BTreeMap<i32, i32>,
    tables: &GameTables,
) -> Result<CraftOutcome, CraftError> {
    let remains = consume_item_costs(&mut player.items, &costs, |item_id, needed, available| {
        CraftError::InsufficientMaterial {
            item_id,
            needed,
            available,
        }
    })?;
    let target = player
        .add_item(target_id, amount, tables)
        .ok_or(CraftError::UnknownTarget(target_id))?;
    Ok(CraftOutcome {
        id,
        amount,
        remains,
        target,
    })
}
#[cfg(test)]
mod tests;
