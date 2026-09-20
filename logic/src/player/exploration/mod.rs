use super::*;
use protocol::pbcommon::{DcNetDataRegion, DcNetDataTaskGroupStatus};

impl Player {
    pub fn gold_coin_ids(&self, city_id: i32) -> Vec<String> {
        self.gold_coins
            .iter()
            .filter(|coin| coin.city_id == city_id)
            .map(|coin| coin.coin_id.clone())
            .collect()
    }

    pub fn collect_gold_coin(
        &mut self,
        city_id: i32,
        coin_id: &str,
        tables: &GameTables,
    ) -> Result<DcNetDataTakeRewardRes, GoldCoinError> {
        let amount = tables
            .gold_coins
            .get(coin_id)
            .ok_or_else(|| GoldCoinError::Unknown(coin_id.to_owned()))?
            .num;
        if self.gold_coins.iter().any(|coin| coin.coin_id == coin_id) {
            return Err(GoldCoinError::AlreadyCollected(coin_id.to_owned()));
        }
        let item = self
            .add_item(tables.cultivation_constants.coin_item_id, amount, tables)
            .ok_or(GoldCoinError::MissingRewardItem)?;
        self.gold_coins.push(CollectedGoldCoin {
            city_id,
            coin_id: coin_id.to_owned(),
        });
        Ok(DcNetDataTakeRewardRes {
            items: vec![item],
            ..Default::default()
        })
    }

    pub fn collection_resource_info(
        &self,
        city_id: i32,
        tables: &GameTables,
    ) -> HashMap<String, i32> {
        tables
            .collection_resources
            .rows
            .iter()
            .filter(|resource| resource.city_id == city_id)
            .map(|resource| {
                let remaining = self
                    .collection_resources
                    .iter()
                    .find(|state| state.collection_id == resource.collection_id)
                    .map_or(resource.collection_num, |state| state.remaining);
                (resource.collection_id.clone(), remaining)
            })
            .collect()
    }

    pub fn collect_collection_resource(
        &mut self,
        collection_id: &str,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, CollectionResourceError> {
        let resource = tables
            .collection_resources
            .get(collection_id)
            .ok_or_else(|| CollectionResourceError::Unknown(collection_id.to_owned()))?;
        let reward_id = resource
            .drop_id
            .ok_or_else(|| CollectionResourceError::MissingReward(collection_id.to_owned()))?;
        let rewards = tables
            .rewards
            .get(reward_id)
            .ok_or_else(|| CollectionResourceError::MissingReward(collection_id.to_owned()))?
            .reward
            .clone();

        let state_index = self
            .collection_resources
            .iter()
            .position(|state| state.collection_id == collection_id);
        let remaining = state_index.map_or(resource.collection_num, |index| {
            self.collection_resources[index].remaining
        });
        if remaining <= 0 {
            return Err(CollectionResourceError::Depleted(collection_id.to_owned()));
        }
        if let Some(index) = state_index {
            self.collection_resources[index].remaining -= 1;
        } else {
            self.collection_resources.push(CollectionResourceState {
                collection_id: collection_id.to_owned(),
                remaining: remaining - 1,
            });
        }

        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn monster_point_groups(
        &mut self,
        city_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> (Vec<DcNetDataMonsterPointGroup>, bool) {
        let reset = self.refresh_daily_world(now);
        let groups = tables
            .main_city_monster_groups
            .rows
            .iter()
            .filter(|group| group.city_id == city_id)
            .map(|group| self.monster_point_group(&group.group_id, tables))
            .collect();
        (groups, reset)
    }

    pub fn defeat_monster_point(
        &mut self,
        point_id: &str,
        tables: &GameTables,
        now: i32,
    ) -> Result<
        (
            DcNetDataTakeRewardRes,
            DcNetDataMonsterPointGroup,
            Vec<DcNetDataRegionCoinLimit>,
        ),
        MonsterPointError,
    > {
        self.refresh_daily_world(now);
        let point = tables
            .main_city_monster_points
            .get(point_id)
            .ok_or_else(|| MonsterPointError::Unknown(point_id.to_owned()))?;
        let group = tables
            .main_city_monster_groups
            .get(&point.group_id)
            .ok_or_else(|| MonsterPointError::UnknownGroup(point_id.to_owned()))?;
        let region = tables
            .city_stages
            .get(group.city_id)
            .and_then(|stage| tables.city_regions.get(stage.region))
            .filter(|region| region.reputation_money != 0);
        if self
            .defeated_monster_points
            .iter()
            .any(|defeated| defeated == point_id)
        {
            let limits = region
                .map(|region| {
                    let amount = self
                        .region_coin_daily
                        .get(&region.reputation_money)
                        .copied()
                        .unwrap_or_default();
                    DcNetDataRegionCoinLimit {
                        coin_id: region.reputation_money,
                        num: amount,
                        limit: region.money_daily_limit,
                        reach_limit: region.money_daily_limit > 0
                            && amount >= region.money_daily_limit,
                    }
                })
                .into_iter()
                .collect();
            return Ok((
                DcNetDataTakeRewardRes::default(),
                self.monster_point_group(&point.group_id, tables),
                limits,
            ));
        }
        let monster = tables.monsters.get(&point.monster_id).ok_or_else(|| {
            MonsterPointError::UnknownMonster {
                point_id: point_id.to_owned(),
                monster_id: point.monster_id.clone(),
            }
        })?;
        let drop = tables.rewards.get(monster.drop_reward).ok_or_else(|| {
            MonsterPointError::MissingMonsterReward {
                monster_id: monster.id.clone(),
                reward_id: monster.drop_reward,
            }
        })?;
        if let Some(region) = region
            && tables.items.get(region.reputation_money).is_none()
        {
            return Err(MonsterPointError::MissingRegionCoin(
                region.reputation_money,
            ));
        }

        self.defeated_monster_points.push(point_id.to_owned());
        let group = self.monster_point_group(&point.group_id, tables);
        let mut reward = DcNetDataTakeRewardRes::default();
        let mut limits = Vec::new();
        if let Some(region) = region {
            let coin_reward = drop
                .reward
                .iter()
                .filter(|entry| entry.key == region.reputation_money)
                .map(|entry| entry.value)
                .sum::<i32>()
                .max(0);
            let current = self
                .region_coin_daily
                .get(&region.reputation_money)
                .copied()
                .unwrap_or_default();
            let available = if region.money_daily_limit > 0 {
                region.money_daily_limit.saturating_sub(current).max(0)
            } else {
                coin_reward
            };
            let granted = coin_reward.min(available);
            let amount = current.saturating_add(granted);
            self.region_coin_daily
                .insert(region.reputation_money, amount);
            if granted > 0 {
                reward.items.push(
                    self.add_item(region.reputation_money, granted, tables)
                        .ok_or(MonsterPointError::MissingRegionCoin(
                            region.reputation_money,
                        ))?,
                );
            }
            limits.push(DcNetDataRegionCoinLimit {
                coin_id: region.reputation_money,
                num: amount,
                limit: region.money_daily_limit,
                reach_limit: region.money_daily_limit > 0 && amount >= region.money_daily_limit,
            });
        }
        Ok((reward, group, limits))
    }

    pub fn region_coin_amount(&self, coin_id: i32) -> i32 {
        self.region_coin_daily
            .get(&coin_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn region_info(
        &self,
        region_id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Option<DcNetDataRegion> {
        let region = tables.city_regions.get(region_id)?;
        let amount = self.region_coin_amount(region.reputation_money);
        let tasks = select_region_tasks(self, region_id, tables, now, zone_offset)
            .into_iter()
            .map(|id| self.region_task_group_status(id, tables, now, zone_offset))
            .collect();

        Some(DcNetDataRegion {
            id: region.id,
            remain_sec: common::time::next_daily_refresh_seconds(
                now,
                zone_offset,
                tables.cultivation_constants.daily_reset_hour,
            ),
            tasks,
            level: self.region_reputation_level(region.reputation_exp, tables),
            region_coin: Some(DcNetDataRegionCoinLimit {
                coin_id: region.reputation_money,
                num: amount,
                limit: region.money_daily_limit,
                reach_limit: region.reputation_money == 0
                    || (region.money_daily_limit > 0 && amount >= region.money_daily_limit),
            }),
        })
    }

    fn region_reputation_level(&self, item_id: i32, tables: &GameTables) -> i32 {
        if item_id == 0 {
            return 1;
        }
        if let Some(region) = tables
            .city_regions
            .rows
            .iter()
            .find(|region| region.reputation_exp == item_id)
            && let Some(&level) = self.region_levels.get(&region.id)
        {
            return level;
        }
        let mut exp = self
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .map_or(0, |item| item.amount.max(0));
        let mut level = 1;
        for config in &tables.region_reputation_levels {
            if config.id != level || config.exp <= 0 || exp < config.exp {
                break;
            }
            exp -= config.exp;
            level += 1;
        }
        level
    }

    pub(super) fn add_region_exp(
        &mut self,
        region_id: i32,
        amount: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataItem, RegionError> {
        let region = tables
            .city_regions
            .get(region_id)
            .ok_or(RegionError::UnknownRegion(region_id))?;
        let mut level = self.region_reputation_level(region.reputation_exp, tables);
        let index = if let Some(index) = self
            .items
            .iter()
            .position(|item| item.item_id == region.reputation_exp)
        {
            index
        } else {
            self.items.push(DcNetDataItem {
                user_item_id: make_entity_id(self.uid, region.reputation_exp),
                item_id: region.reputation_exp,
                quality: tables
                    .items
                    .get(region.reputation_exp)
                    .map_or(0, |item| item.quality),
                ..Default::default()
            });
            self.items.len() - 1
        };
        let mut exp = self.items[index].amount.saturating_add(amount.max(0));
        while let Some(config) = tables
            .region_reputation_levels
            .iter()
            .find(|config| config.id == level)
        {
            if config.exp <= 0 || exp < config.exp {
                break;
            }
            exp -= config.exp;
            level += 1;
        }
        self.items[index].amount = exp;
        self.region_levels.insert(region_id, level);
        Ok(self.items[index])
    }

    pub fn is_region_ticket_taken(&self, tables: &GameTables, now: i32, zone_offset: i32) -> bool {
        self.region_ticket_day
            == common::time::daily_refresh_day(
                now,
                zone_offset,
                tables.cultivation_constants.daily_reset_hour,
            )
    }

    pub fn claim_region_daily_ticket(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataItem, RegionError> {
        if self.is_region_ticket_taken(tables, now, zone_offset) {
            return Err(RegionError::DailyTicketClaimed);
        }
        let item_id = tables
            .region_tasks_by_region
            .rows
            .first()
            .map(|row| row.count.key)
            .ok_or(RegionError::MissingTicketConfig)?;
        let current = self
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .map_or(0, |item| item.amount);
        let available = tables
            .cultivation_constants
            .region_ticket_limit
            .saturating_sub(current);
        if available <= 0 {
            return Err(RegionError::TicketFull);
        }
        let amount = tables
            .cultivation_constants
            .daily_region_tickets
            .max(0)
            .min(available);
        let item = self
            .add_item(item_id, amount, tables)
            .ok_or(RegionError::MissingTicketConfig)?;
        self.region_ticket_day = common::time::daily_refresh_day(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        Ok(item)
    }

    pub fn accept_region_task(
        &mut self,
        region_id: i32,
        group_id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataTaskGroupStatus, Vec<DcNetDataItem>), RegionError> {
        if tables.city_regions.get(region_id).is_none() {
            return Err(RegionError::UnknownRegion(region_id));
        }
        let row = tables
            .region_tasks_by_region
            .get(region_id)
            .and_then(|mut rows| rows.find(|row| row.task_group_id == group_id))
            .cloned()
            .ok_or(RegionError::UnknownTaskGroup {
                region_id,
                group_id,
            })?;
        if !select_region_tasks(self, region_id, tables, now, zone_offset).contains(&group_id) {
            return Err(RegionError::NotOffered(group_id));
        }
        if self
            .region_task_group_status(group_id, tables, now, zone_offset)
            .status
            != TaskStatus::Disabled as i32
        {
            return Err(RegionError::AlreadyAccepted(group_id));
        }

        let mut active_groups = Vec::new();
        for task in self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Picked as i32)
        {
            let Some(config) = tables.tasks.get(task.id) else {
                continue;
            };
            if tables
                .region_tasks_by_region
                .rows
                .iter()
                .any(|row| row.task_group_id == config.task_group)
                && !active_groups.contains(&config.task_group)
            {
                active_groups.push(config.task_group);
            }
        }
        if active_groups.len() >= tables.cultivation_constants.max_region_tasks.max(0) as usize {
            return Err(RegionError::ActiveLimit);
        }

        let config = tables
            .tasks
            .rows
            .iter()
            .filter(|task| task.task_group == group_id && task.sort > 0)
            .min_by_key(|task| (task.sort, task.id))
            .cloned()
            .ok_or(RegionError::EmptyTaskGroup(group_id))?;
        let cost = row.count.value.max(0);
        let item_index = self
            .items
            .iter()
            .position(|item| item.item_id == row.count.key);
        let available = item_index.map_or(0, |index| self.items[index].amount);
        if available < cost {
            return Err(RegionError::InsufficientTickets {
                item_id: row.count.key,
                needed: cost,
                available,
            });
        }
        let remains = if let Some(index) = item_index {
            self.items[index].amount -= cost;
            vec![self.items[index]]
        } else {
            Vec::new()
        };
        let total = config
            .done_key
            .first()
            .and_then(|limit| limit.value.rsplit('|').next())
            .and_then(|total| total.parse().ok())
            .unwrap_or_default();
        let task = DcNetDataTaskStatus {
            id: config.id,
            status: TaskStatus::Picked as i32,
            picked_at: i64::from(now),
            progress: 0,
            total,
        };
        self.store_task_status(task);
        Ok((
            DcNetDataTaskGroupStatus {
                id: group_id,
                status: TaskStatus::Picked as i32,
                curr_task: Some(task),
            },
            remains,
        ))
    }

    fn region_task_group_status(
        &self,
        group_id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> DcNetDataTaskGroupStatus {
        let day = common::time::daily_refresh_day(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        let mut current = None;
        let mut completed_today = false;
        for task in &self.tasks {
            let Some(config) = tables.tasks.get(task.id) else {
                continue;
            };
            if config.task_group != group_id {
                continue;
            }
            if task.status == TaskStatus::Picked as i32
                && current.is_none_or(|(_, sort)| config.sort < sort)
            {
                current = Some((*task, config.sort));
            }
            completed_today |= task.status == TaskStatus::Done as i32
                && common::time::daily_refresh_day(
                    i32::try_from(task.picked_at).unwrap_or_default(),
                    zone_offset,
                    tables.cultivation_constants.daily_reset_hour,
                ) == day;
        }
        DcNetDataTaskGroupStatus {
            id: group_id,
            status: if current.is_some() {
                TaskStatus::Picked as i32
            } else if completed_today {
                TaskStatus::Done as i32
            } else {
                TaskStatus::Disabled as i32
            },
            curr_task: current.map(|(task, _)| task),
        }
    }

    pub fn reward_boxes(&self) -> Vec<DcNetDataTBoxInfo> {
        self.reward_boxes.clone()
    }

    pub fn achievements(&self, tables: &GameTables, now: i32) -> Vec<DcNetDataAchi> {
        tables
            .achievements
            .rows
            .iter()
            .filter_map(|achievement| self.achievement_info(achievement.id, tables, now))
            .collect()
    }

    pub fn claim_achievement(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataAchi, DcNetDataTakeRewardRes), AchievementError> {
        let rewards = tables
            .achievements
            .get(id)
            .ok_or(AchievementError::Unknown(id))?
            .achieve_award
            .clone();
        let mut achievement = self
            .achievement_info(id, tables, now)
            .ok_or(AchievementError::Unknown(id))?;
        if achievement.took_at > 0 {
            return Err(AchievementError::AlreadyClaimed(id));
        }
        if achievement.progress < achievement.total {
            return Err(AchievementError::Incomplete(id));
        }

        achievement.took_at = i64::from(now);
        self.achievements.push(achievement);
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok((achievement, result))
    }

    fn achievement_info(&self, id: i32, tables: &GameTables, now: i32) -> Option<DcNetDataAchi> {
        let total = tables.achievement_goals.get(id)?.total;
        let chain = tables.achievement_chains.get(id / 100)?;
        let progress = chain
            .achieve_finish_limit
            .first()
            .map_or(0, |condition| {
                self.achievement_progress(condition.key, &condition.value, tables, now)
            })
            .min(total);
        let took_at = self
            .achievements
            .iter()
            .find(|achievement| achievement.id == id)
            .map_or(0, |achievement| achievement.took_at);
        Some(DcNetDataAchi {
            id,
            took_at,
            progress,
            total,
        })
    }

    pub fn completed_achievements_since(
        &self,
        before: &[DcNetDataAchi],
        tables: &GameTables,
        now: i32,
    ) -> Vec<DcNetDataAchi> {
        self.achievements(tables, now)
            .into_iter()
            .filter(|current| current.progress >= current.total)
            .filter(|current| {
                before
                    .iter()
                    .find(|previous| previous.id == current.id)
                    .is_none_or(|previous| previous.progress < previous.total)
            })
            .collect()
    }

    fn achievement_progress(
        &self,
        condition: i32,
        value: &str,
        tables: &GameTables,
        now: i32,
    ) -> i32 {
        match condition {
            // Achievement condition 50 tracks lifetime acquisition. Client-side
            // limit checks with the same key still use the current inventory.
            50 => value
                .split('|')
                .next()
                .and_then(|id| id.parse::<i32>().ok())
                .and_then(|id| self.item_acquired.get(&id).copied())
                .unwrap_or_default(),
            // The base album is always selected; progress begins when the player
            // adds a user-selectable album beside it (captured achievement 700201).
            93 => self
                .albums
                .iter()
                .filter(|album| album.status == AlbumStatus::Selected as i32)
                .count()
                .saturating_sub(1) as i32,
            // Achievement key 202 is current equipment ownership. Battle-pass
            // tasks reuse the key for a period-scoped acquisition counter.
            202 => self.equips.len() as i32,
            _ => self.condition_progress(condition, value, tables, now),
        }
    }
}

fn select_region_tasks(
    player: &Player,
    region_id: i32,
    tables: &GameTables,
    now: i32,
    zone_offset: i32,
) -> Vec<i32> {
    let Some(rows) = tables.region_tasks_by_region.get(region_id) else {
        return Vec::new();
    };
    let mut candidates: Vec<_> = rows
        .filter(|row| {
            row.weight > 0
                && row.limit_type.iter().all(|limit| {
                    player.condition_progress(limit.key, &limit.value, tables, now) > 0
                })
        })
        .collect();
    candidates.sort_by_key(|row| row.task_group_id);

    let count = tables.cultivation_constants.max_region_tasks.max(0) as usize;
    let type_min = tables.cultivation_constants.min_region_task_types.max(0) as usize;
    let day = common::time::daily_refresh_day(
        now,
        zone_offset,
        tables.cultivation_constants.daily_reset_hour,
    );
    let mut state =
        (player.uid as u64) ^ (day as u64).rotate_left(21) ^ (region_id as u64).rotate_left(43);
    let mut selected = Vec::with_capacity(count.min(candidates.len()));
    let mut selected_types = Vec::new();
    while selected.len() < count && !candidates.is_empty() {
        let require_new_type = selected_types.len() < type_min;
        let total: u64 = candidates
            .iter()
            .filter(|row| !require_new_type || !selected_types.contains(&row.task_type))
            .map(|row| row.weight as u64)
            .sum();
        if total == 0 {
            break;
        }
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut roll = mix_region_seed(state) % total;
        let index = candidates
            .iter()
            .position(|row| {
                if require_new_type && selected_types.contains(&row.task_type) {
                    return false;
                }
                let weight = row.weight as u64;
                if roll < weight {
                    true
                } else {
                    roll -= weight;
                    false
                }
            })
            .unwrap_or_default();
        let row = candidates.remove(index);
        if !selected_types.contains(&row.task_type) {
            selected_types.push(row.task_type);
        }
        selected.push(row.task_group_id);
    }
    selected
}

fn mix_region_seed(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
#[cfg(test)]
mod tests;
