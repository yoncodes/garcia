use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemUseOutcome {
    pub rewards: DcNetDataTakeRewardRes,
    pub remain: DcNetDataItem,
}

impl Player {
    pub fn grant_reward(
        &mut self,
        reward_id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, InventoryError> {
        validate_rewards(&[(reward_id, amount)], tables)?;
        let mut rewards = DcNetDataTakeRewardRes::default();
        self.add_reward(reward_id, amount, tables, now, &mut rewards);
        Ok(rewards)
    }

    pub fn use_item(
        &mut self,
        request: DcNetDataUseItem,
        tables: &GameTables,
        now: i32,
    ) -> Result<ItemUseOutcome, InventoryError> {
        let index = self.find_usable_item_index(request)?;
        let config = tables
            .items
            .get(request.item_id)
            .ok_or(InventoryError::UnknownItem(request.item_id))?;
        let effect = config
            .effect
            .as_ref()
            .ok_or(InventoryError::MissingEffect(request.item_id))?;
        if request.amount <= 0 {
            return Err(InventoryError::InvalidAmount(request.amount));
        }

        let mut rewards = match config.item_type {
            2 | 101 => vec![(effect.key, checked_amount(effect.value, request.amount)?)],
            102 => package_rewards(
                effect.key,
                checked_amount(effect.value, request.amount)?,
                config.sub_type == 11,
                tables,
            )?
            .into_iter()
            .collect(),
            103 => return Err(InventoryError::SelectablePackage(request.item_id)),
            kind => return Err(InventoryError::UnsupportedType(kind)),
        };
        for (id, amount) in &mut rewards {
            if *id == tables.cultivation_constants.heat_item_id {
                let current = self
                    .items
                    .iter()
                    .find(|item| item.item_id == *id)
                    .map_or(0, |item| item.amount);
                *amount = (*amount).min(
                    tables
                        .cultivation_constants
                        .heat_limit
                        .saturating_sub(current),
                );
                if *amount <= 0 {
                    return Err(InventoryError::HeatFull);
                }
            }
        }
        validate_rewards(&rewards, tables)?;
        ensure_available(&self.items[index], request.amount)?;

        self.items[index].amount -= request.amount;
        let remain = self.items[index];
        let mut result = DcNetDataTakeRewardRes::default();
        for (id, amount) in rewards {
            self.add_reward(id, amount, tables, now, &mut result);
        }
        Ok(ItemUseOutcome {
            rewards: result,
            remain,
        })
    }

    pub fn select_package_reward(
        &mut self,
        item_id: i32,
        selections: &[(i32, i32)],
        tables: &GameTables,
        now: i32,
    ) -> Result<ItemUseOutcome, InventoryError> {
        let config = tables
            .items
            .get(item_id)
            .ok_or(InventoryError::UnknownItem(item_id))?;
        if config.item_type != 103 {
            return Err(InventoryError::NotSelectablePackage(item_id));
        }
        let reward_id = config
            .effect
            .as_ref()
            .ok_or(InventoryError::MissingEffect(item_id))?
            .key;
        let available_rewards = reward_counts(reward_id, tables)?;
        let mut selected = BTreeMap::new();
        let mut used = 0i32;
        for &(id, amount) in selections {
            if amount <= 0 {
                return Err(InventoryError::InvalidAmount(amount));
            }
            if !available_rewards.contains_key(&id) {
                return Err(InventoryError::InvalidSelection(id));
            }
            used = used
                .checked_add(amount)
                .ok_or(InventoryError::InvalidAmount(amount))?;
            *selected.entry(id).or_default() += amount;
        }
        if used == 0 {
            return Err(InventoryError::EmptySelection);
        }
        let index = self
            .items
            .iter()
            .position(|item| item.item_id == item_id)
            .ok_or(InventoryError::UnknownItem(item_id))?;
        ensure_available(&self.items[index], used)?;

        let rewards = selected
            .into_iter()
            .map(|(id, picks)| {
                checked_amount(available_rewards[&id], picks).map(|amount| (id, amount))
            })
            .collect::<Result<Vec<_>, _>>()?;
        validate_rewards(&rewards, tables)?;
        self.items[index].amount -= used;
        let remain = self.items[index];
        let mut result = DcNetDataTakeRewardRes::default();
        for (id, amount) in rewards {
            self.add_reward(id, amount, tables, now, &mut result);
        }
        Ok(ItemUseOutcome {
            rewards: result,
            remain,
        })
    }

    fn find_usable_item_index(&self, request: DcNetDataUseItem) -> Result<usize, InventoryError> {
        self.items
            .iter()
            .position(|item| {
                item.item_id == request.item_id
                    && (request.user_item_id == 0 || item.user_item_id == request.user_item_id)
            })
            .ok_or(InventoryError::UnknownItem(request.item_id))
    }
}

fn package_rewards(
    reward_id: i32,
    copies: i32,
    random_relic: bool,
    tables: &GameTables,
) -> Result<BTreeMap<i32, i32>, InventoryError> {
    let leaves = reward_leaves(reward_id, tables)?;
    if leaves.is_empty() {
        return Err(InventoryError::UnknownReward(reward_id));
    }
    if !random_relic {
        return reward_counts(reward_id, tables)?
            .into_iter()
            .map(|(id, amount)| checked_amount(amount, copies).map(|amount| (id, amount)))
            .collect();
    }
    if !leaves.iter().all(|id| tables.equipment.get(*id).is_some()) {
        return Err(InventoryError::InvalidRelicPackage(reward_id));
    }
    let mut counts = BTreeMap::new();
    for _ in 0..copies {
        let id = leaves[fastrand::usize(..leaves.len())];
        *counts.entry(id).or_default() += 1;
    }
    Ok(counts)
}

fn reward_counts(
    reward_id: i32,
    tables: &GameTables,
) -> Result<BTreeMap<i32, i32>, InventoryError> {
    let leaves = reward_leaves(reward_id, tables)?;
    let mut counts = BTreeMap::new();
    for id in leaves {
        *counts.entry(id).or_default() += 1;
    }
    Ok(counts)
}

fn reward_leaves(reward_id: i32, tables: &GameTables) -> Result<Vec<i32>, InventoryError> {
    if tables.rewards.get(reward_id).is_none() {
        return Err(InventoryError::UnknownReward(reward_id));
    }
    let mut leaves = Vec::new();
    collect_reward_leaves(reward_id, tables, &mut leaves, 0);
    Ok(leaves)
}

fn validate_rewards(rewards: &[(i32, i32)], tables: &GameTables) -> Result<(), InventoryError> {
    for &(id, amount) in rewards {
        if amount <= 0 {
            return Err(InventoryError::InvalidAmount(amount));
        }
        let known = tables.items.get(id).is_some()
            || tables.maids.get(id).is_some()
            || tables.partners.get(id).is_some()
            || tables.equipment.get(id).is_some()
            || tables.collections.get(id).is_some()
            || tables.maid_skins.get(id).is_some()
            || tables.profile_avatars.get(id).is_some()
            || tables.profile_frames.get(id).is_some()
            || tables.profile_cards.get(id).is_some()
            || tables.profile_titles.get(id).is_some();
        if !known {
            return Err(InventoryError::UnknownReward(id));
        }
    }
    Ok(())
}

fn checked_amount(value: i32, multiplier: i32) -> Result<i32, InventoryError> {
    value
        .checked_mul(multiplier)
        .filter(|amount| *amount > 0)
        .ok_or(InventoryError::InvalidAmount(multiplier))
}

fn ensure_available(item: &DcNetDataItem, needed: i32) -> Result<(), InventoryError> {
    if item.amount < needed {
        return Err(InventoryError::InsufficientItem {
            item_id: item.item_id,
            needed,
            available: item.amount,
        });
    }
    Ok(())
}
#[cfg(test)]
mod tests;
