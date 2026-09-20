use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattlePassState {
    pub id: i32,
    pub paid_status: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_week: i32,
    pub week: i32,
    pub rewards: BTreeMap<i32, i32>,
    pub tasks: Vec<i32>,
}

pub struct BattlePassInfo {
    pub id: i32,
    pub exp_week_limit: i32,
    pub exp_per_level: i32,
    pub level_max: i32,
    pub start_time: String,
    pub end_time: String,
    pub advanced_reward: BTreeMap<i32, i32>,
    pub premium_reward: BTreeMap<i32, i32>,
    pub battle_pass: DcNetDataBattlePass,
}

impl Player {
    pub fn battle_pass_info(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Option<(BattlePassInfo, bool)> {
        let (config, changed) = self.sync_battle_pass(tables, now, zone_offset).ok()?;
        Some((
            BattlePassInfo {
                id: config.id,
                exp_week_limit: config.exp_week_limit,
                exp_per_level: config.exp_per_level,
                level_max: config.level_max,
                start_time: config.start_time.clone(),
                end_time: config.end_time.clone(),
                advanced_reward: config
                    .advanced_reward
                    .iter()
                    .map(|reward| (reward.key, reward.value))
                    .collect(),
                premium_reward: config
                    .premium_reward
                    .iter()
                    .map(|reward| (reward.key, reward.value))
                    .collect(),
                battle_pass: self.battle_pass_data(&config, now, zone_offset),
            },
            changed,
        ))
    }

    pub fn battle_pass_tasks(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Option<(Vec<DcNetDataBattlePassTask>, i32, i32, bool)> {
        let (config, changed) = self.sync_battle_pass(tables, now, zone_offset).ok()?;
        let tasks = visible_battle_pass_tasks(&config, &self.battle_pass, tables)
            .into_iter()
            .map(|task| self.battle_pass_task_data(task, tables, now))
            .collect();
        Some((
            tasks,
            common::time::next_weekly_refresh_seconds(
                now,
                zone_offset,
                tables.cultivation_constants.daily_reset_hour,
            ),
            common::time::next_daily_refresh_seconds(
                now,
                zone_offset,
                tables.cultivation_constants.daily_reset_hour,
            ),
            changed,
        ))
    }

    pub(super) fn battle_pass_task_snapshot(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataBattlePassTask> {
        self.battle_pass_tasks(tables, now, zone_offset)
            .map(|(tasks, _, _, _)| tasks)
            .unwrap_or_default()
    }

    pub(super) fn changed_battle_pass_tasks_since(
        &mut self,
        before: &[DcNetDataBattlePassTask],
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataBattlePassTask> {
        self.battle_pass_task_snapshot(tables, now, zone_offset)
            .into_iter()
            .filter(|current| {
                before
                    .iter()
                    .find(|previous| previous.id == current.id)
                    .is_some_and(|previous| previous.progress != current.progress)
            })
            .collect()
    }

    pub fn battle_pass_reward_updates(
        &mut self,
        reward: &DcNetDataTakeRewardRes,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataBattlePassTask> {
        let Ok((config, _)) = self.sync_battle_pass(tables, now, zone_offset) else {
            return Vec::new();
        };
        visible_battle_pass_tasks(&config, &self.battle_pass, tables)
            .into_iter()
            .filter_map(|task| {
                let limit = task.finish_limit.first()?;
                let item_id = limit.value.split('|').next()?.parse::<i32>().ok()?;
                (limit.key == 50 && reward.items.iter().any(|item| item.item_id == item_id)).then(
                    || {
                        let mut update = self.battle_pass_task_data(task, tables, now);
                        update.progress =
                            self.condition_progress(limit.key, &limit.value, tables, now);
                        update
                    },
                )
            })
            .collect()
    }

    pub fn battle_pass_login_updates(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataBattlePassTask> {
        let Ok((config, _)) = self.sync_battle_pass(tables, now, zone_offset) else {
            return Vec::new();
        };
        visible_battle_pass_tasks(&config, &self.battle_pass, tables)
            .into_iter()
            .filter(|task| {
                task.finish_limit
                    .first()
                    .is_some_and(|limit| matches!(limit.key, 3 | 4))
            })
            .map(|task| self.battle_pass_task_data(task, tables, now))
            .collect()
    }

    pub fn claim_battle_pass_tasks(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(Vec<DcNetDataBattlePassTask>, DcNetDataBattlePass), BattlePassError> {
        let (config, _) = self.sync_battle_pass(tables, now, zone_offset)?;
        let candidates: Vec<_> = visible_battle_pass_tasks(&config, &self.battle_pass, tables)
            .into_iter()
            .filter(|task| id == 0 || task.id == id)
            .cloned()
            .collect();
        if id != 0 && candidates.is_empty() {
            if self.battle_pass.tasks.contains(&id) {
                return Err(BattlePassError::TaskAlreadyClaimed(id));
            }
            return Err(BattlePassError::UnknownTask(id));
        }

        let mut claimed = Vec::new();
        for task in candidates {
            if self.battle_pass.tasks.contains(&task.id) {
                if id != 0 {
                    return Err(BattlePassError::TaskAlreadyClaimed(task.id));
                }
                continue;
            }
            let data = self.battle_pass_task_data(&task, tables, now);
            if data.progress < data.total {
                if id != 0 {
                    return Err(BattlePassError::IncompleteTask(task.id));
                }
                continue;
            }
            self.battle_pass.tasks.push(task.id);
            self.add_battle_pass_exp(task.exp.value, &config);
            claimed.push(DcNetDataBattlePassTask {
                status: TaskStatus::Done as i32,
                ..data
            });
        }
        self.battle_pass.tasks.sort_unstable();
        Ok((claimed, self.battle_pass_data(&config, now, zone_offset)))
    }

    pub fn claim_battle_pass_rewards(
        &mut self,
        level: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataTakeRewardRes, DcNetDataBattlePass), BattlePassError> {
        let (config, _) = self.sync_battle_pass(tables, now, zone_offset)?;
        if level != 0
            && tables
                .battle_pass_rewards
                .iter()
                .all(|reward| reward.id != level)
        {
            return Err(BattlePassError::UnknownReward(level));
        }
        if level > self.battle_pass.level {
            return Err(BattlePassError::LockedReward(level));
        }

        let target_state = if self.battle_pass.paid_status > PaidStatus::Free as i32 {
            2
        } else {
            1
        };
        let rewards: Vec<_> = tables
            .battle_pass_rewards
            .iter()
            .filter(|reward| reward.id <= self.battle_pass.level)
            .filter(|reward| level == 0 || reward.id == level)
            .cloned()
            .collect();
        if level != 0
            && self
                .battle_pass
                .rewards
                .get(&level)
                .copied()
                .unwrap_or_default()
                >= target_state
        {
            return Err(BattlePassError::RewardAlreadyClaimed(level));
        }

        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            let previous = self
                .battle_pass
                .rewards
                .get(&reward.id)
                .copied()
                .unwrap_or_default();
            if previous < 1 {
                self.add_reward(
                    reward.base_reward.key,
                    reward.base_reward.value,
                    tables,
                    now,
                    &mut result,
                );
            }
            if previous < 2 && target_state == 2 {
                for extra in reward.extra_reward {
                    self.add_reward(extra.key, extra.value, tables, now, &mut result);
                }
            }
            self.battle_pass.rewards.insert(reward.id, target_state);
        }
        Ok((result, self.battle_pass_data(&config, now, zone_offset)))
    }

    pub fn level_up_battle_pass(
        &mut self,
        add_level: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataItem, DcNetDataBattlePass), BattlePassError> {
        let (config, _) = self.sync_battle_pass(tables, now, zone_offset)?;
        if add_level <= 0 {
            return Err(BattlePassError::InvalidLevelIncrease);
        }
        if self.battle_pass.level.saturating_add(add_level) > config.level_max {
            return Err(BattlePassError::LevelCap(config.level_max));
        }
        let currency_item_id = tables.cultivation_constants.diamond_item_id;
        let needed = tables
            .cultivation_constants
            .battle_pass_level_price
            .saturating_mul(add_level);
        let available = self
            .items
            .iter()
            .find(|item| item.item_id == currency_item_id)
            .map_or(0, |item| item.amount);
        if available < needed {
            return Err(BattlePassError::InsufficientCost {
                item_id: currency_item_id,
                needed,
                available,
            });
        }
        let item = self
            .items
            .iter_mut()
            .find(|item| item.item_id == currency_item_id)
            .expect("currency was checked above");
        item.amount -= needed;
        let remain = *item;
        self.battle_pass.level += add_level;
        Ok((remain, self.battle_pass_data(&config, now, zone_offset)))
    }

    fn sync_battle_pass(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(configs::tables::BattlePass, bool), BattlePassError> {
        let config = tables
            .battle_passes
            .rows
            .iter()
            .find(|config| {
                common::time::table_window_active(
                    &config.start_time,
                    &config.end_time,
                    now,
                    zone_offset,
                )
            })
            .cloned()
            .ok_or(BattlePassError::Inactive)?;
        let week = common::time::weekly_refresh_index(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        let mut changed = false;
        if self.battle_pass.id != config.id {
            self.battle_pass = BattlePassState {
                id: config.id,
                paid_status: PaidStatus::Free as i32,
                level: 1,
                week,
                ..Default::default()
            };
            changed = true;
        } else if self.battle_pass.week != week {
            self.battle_pass.week = week;
            self.battle_pass.exp_week = 0;
            changed = true;
        }
        Ok((config, changed))
    }

    fn battle_pass_task_data(
        &self,
        task: &configs::tables::BattlePassTask,
        tables: &GameTables,
        now: i32,
    ) -> DcNetDataBattlePassTask {
        let limit = task.finish_limit.first();
        let total = limit.map_or(0, |limit| condition_total(limit.key, &limit.value));
        DcNetDataBattlePassTask {
            id: task.id,
            status: if self.battle_pass.tasks.contains(&task.id) {
                TaskStatus::Done as i32
            } else {
                TaskStatus::Disabled as i32
            },
            progress: limit.map_or(0, |limit| {
                self.condition_progress(limit.key, &limit.value, tables, now)
            }),
            total,
        }
    }

    fn add_battle_pass_exp(&mut self, amount: i32, config: &configs::tables::BattlePass) {
        let amount = amount.max(0).min(
            config
                .exp_week_limit
                .saturating_sub(self.battle_pass.exp_week),
        );
        self.battle_pass.exp_week = self.battle_pass.exp_week.saturating_add(amount);
        self.battle_pass.exp = self.battle_pass.exp.saturating_add(amount);
        while self.battle_pass.level < config.level_max
            && self.battle_pass.exp >= config.exp_per_level
            && config.exp_per_level > 0
        {
            self.battle_pass.exp -= config.exp_per_level;
            self.battle_pass.level += 1;
        }
    }

    fn battle_pass_data(
        &self,
        config: &configs::tables::BattlePass,
        now: i32,
        zone_offset: i32,
    ) -> DcNetDataBattlePass {
        let end =
            common::time::table_time_utc(&config.end_time, zone_offset).unwrap_or(i64::from(now));
        DcNetDataBattlePass {
            id: self.battle_pass.id,
            paid_status: self.battle_pass.paid_status,
            remain_sec: common::time::seconds_until(end, i64::from(now)),
            level: self.battle_pass.level,
            exp: self.battle_pass.exp,
            taken: self
                .battle_pass
                .rewards
                .iter()
                .map(|(&k, &v)| (k, v))
                .collect(),
            exp_week: self.battle_pass.exp_week,
        }
    }
}
#[cfg(test)]
mod tests;
