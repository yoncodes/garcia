use super::*;

impl Player {
    pub fn claim_collection_suit_reward(
        &mut self,
        suit_id: i32,
        step: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataTakeRewardRes, Vec<i32>), CollectionError> {
        let suit = tables
            .collection_suits
            .get(suit_id)
            .ok_or(CollectionError::UnknownSuit(suit_id))?;
        let required = suit
            .step
            .iter()
            .find(|entry| entry.key == step)
            .map(|entry| entry.value)
            .ok_or(CollectionError::UnknownStep { suit_id, step })?;
        let reward = suit
            .step_reward
            .iter()
            .find(|entry| entry.key == step)
            .map(|entry| entry.value)
            .ok_or(CollectionError::UnknownStep { suit_id, step })?;
        let owned = self
            .collections
            .iter()
            .filter(|collection| {
                tables
                    .collections
                    .get(collection.cid)
                    .is_some_and(|config| config.suit_id == suit_id)
            })
            .count() as i32;
        if owned < required {
            return Err(CollectionError::SuitIncomplete {
                suit_id,
                step,
                required,
            });
        }

        let index = self
            .collection_suit_rewards
            .iter()
            .position(|state| state.sid == suit_id);
        if index.is_some_and(|index| self.collection_suit_rewards[index].steps.contains(&step)) {
            return Err(CollectionError::SuitRewardClaimed { suit_id, step });
        }
        let state = if let Some(index) = index {
            &mut self.collection_suit_rewards[index]
        } else {
            self.collection_suit_rewards
                .push(DcNetDataCollectionSuitReward {
                    sid: suit_id,
                    steps: Vec::new(),
                });
            self.collection_suit_rewards.last_mut().unwrap()
        };
        state.steps.push(step);
        state.steps.sort_unstable();
        let steps = state.steps.clone();

        let mut result = DcNetDataTakeRewardRes::default();
        self.add_reward(
            tables.cultivation_constants.diamond_item_id,
            reward,
            tables,
            now,
            &mut result,
        );
        Ok((result, steps))
    }

    pub fn claim_collection_reward(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, CollectionError> {
        let reward_id = tables
            .collections
            .get(id)
            .ok_or(CollectionError::UnknownCollection(id))?
            .reward;
        if reward_id <= 0 {
            return Err(CollectionError::MissingReward(id));
        }
        let rewards = tables
            .rewards
            .get(reward_id)
            .ok_or(CollectionError::MissingReward(id))?
            .reward
            .clone();
        let collection = self
            .collections
            .iter_mut()
            .find(|collection| collection.cid == id)
            .ok_or(CollectionError::CollectionNotOwned(id))?;
        if collection.reward {
            return Err(CollectionError::CollectionRewardClaimed(id));
        }
        collection.reward = true;

        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn place_collection(
        &mut self,
        collection_id: i32,
        platform_id: i32,
        tables: &GameTables,
    ) -> Result<Vec<DcNetDataCollectionPlaceInfo>, CollectionError> {
        if platform_id <= 0 {
            return Err(CollectionError::InvalidPlatform(platform_id));
        }
        if collection_id != 0 {
            if tables.collections.get(collection_id).is_none() {
                return Err(CollectionError::UnknownCollection(collection_id));
            }
            if !self
                .collections
                .iter()
                .any(|collection| collection.cid == collection_id)
            {
                return Err(CollectionError::CollectionNotOwned(collection_id));
            }
        }

        let mut changed = Vec::new();
        if collection_id != 0
            && let Some(index) = self
                .collection_places
                .iter()
                .position(|place| place.cid == collection_id && place.pid != platform_id)
        {
            let old = self.collection_places.remove(index);
            changed.push(DcNetDataCollectionPlaceInfo {
                cid: 0,
                pid: old.pid,
            });
        }
        self.collection_places
            .retain(|place| place.pid != platform_id);
        let place = DcNetDataCollectionPlaceInfo {
            cid: collection_id,
            pid: platform_id,
        };
        if collection_id != 0 {
            self.collection_places.push(place);
            self.collection_places.sort_by_key(|place| place.pid);
        }
        changed.push(place);
        Ok(changed)
    }
}
#[cfg(test)]
mod tests;
