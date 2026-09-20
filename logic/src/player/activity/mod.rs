use super::*;

impl Player {
    pub fn active_version_activity(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Option<(i32, i32)> {
        tables.version_activities.rows.iter().find_map(|activity| {
            let countdown = common::time::table_window_remaining(
                &activity.open_time,
                &activity.close_time,
                now,
                zone_offset,
            )?;
            activity
                .limit
                .iter()
                .all(|limit| {
                    self.condition_progress(limit.key, &limit.value, tables, now)
                        >= condition_total(limit.key, &limit.value)
                })
                .then_some((activity.id, countdown))
        })
    }

    pub fn version_tasks(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataVersionTask> {
        let Some((activity_id, _)) = self.active_version_activity(tables, now, zone_offset) else {
            return Vec::new();
        };
        let Some(group) = tables
            .version_activity_task_groups
            .rows
            .iter()
            .find(|group| {
                group.activity_id == activity_id
                    && common::time::table_window_remaining(
                        &group.open_time,
                        &group.close_time,
                        now,
                        zone_offset,
                    )
                    .is_some()
            })
        else {
            return Vec::new();
        };
        tables
            .version_activity_tasks
            .rows
            .iter()
            .filter(|task| task.group_id == group.id)
            .filter_map(|task| self.version_task_info(task.id, tables, now))
            .collect()
    }

    pub fn claim_version_task(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataVersionTask, DcNetDataTakeRewardRes), VersionTaskError> {
        let config = tables
            .version_activity_tasks
            .get(id)
            .ok_or(VersionTaskError::Unknown(id))?;
        let group = tables
            .version_activity_task_groups
            .get(config.group_id)
            .ok_or(VersionTaskError::Inactive)?;
        if self
            .active_version_activity(tables, now, zone_offset)
            .is_none_or(|(activity_id, _)| activity_id != group.activity_id)
            || common::time::table_window_remaining(
                &group.open_time,
                &group.close_time,
                now,
                zone_offset,
            )
            .is_none()
        {
            return Err(VersionTaskError::Inactive);
        }
        if self.version_task_claims.contains(&id) {
            return Err(VersionTaskError::AlreadyClaimed(id));
        }
        let mut task = self
            .version_task_info(id, tables, now)
            .ok_or(VersionTaskError::Unknown(id))?;
        if task.progress < task.total {
            return Err(VersionTaskError::Incomplete(id));
        }

        let rewards = config.reward.clone();
        self.version_task_claims.push(id);
        self.version_task_claims.sort_unstable();
        task.status = TaskStatus::Done as i32;
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok((task, result))
    }

    fn version_task_info(
        &self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Option<DcNetDataVersionTask> {
        let config = tables.version_activity_tasks.get(id)?;
        let limit = config.finish_limit.first()?;
        let total = condition_total(limit.key, &limit.value);
        Some(DcNetDataVersionTask {
            id,
            status: if self.version_task_claims.contains(&id) {
                TaskStatus::Done as i32
            } else {
                TaskStatus::Picked as i32
            },
            progress: self
                .condition_progress(limit.key, &limit.value, tables, now)
                .min(total),
            total,
        })
    }
    pub fn is_limited_level_activity_available(
        &self,
        aid: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> bool {
        tables
            .activities
            .get(aid)
            .is_some_and(|activity| activity.activity_type == 7)
            && self
                .open_activities(tables, now, zone_offset)
                .iter()
                .any(|info| info.aid == aid && info.in_status)
    }

    pub fn claim_limited_level_reward(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataTakeRewardRes, LimitedLevelRewardError> {
        let active = tables.activities.rows.iter().any(|activity| {
            activity.activity_type == 7
                && self.is_limited_level_activity_available(activity.id, tables, now, zone_offset)
        });
        if !active {
            return Err(LimitedLevelRewardError::Inactive);
        }
        let config = tables
            .limited_level_rewards
            .get(id)
            .ok_or(LimitedLevelRewardError::Unknown(id))?;
        if self.level < config.required_level {
            return Err(LimitedLevelRewardError::LevelLocked {
                id,
                required: config.required_level,
            });
        }
        if self.activity_limited_level_rewards.contains(&id) {
            return Err(LimitedLevelRewardError::AlreadyClaimed(id));
        }
        let rewards = config.reward.clone();
        self.activity_limited_level_rewards.push(id);
        self.activity_limited_level_rewards.sort_unstable();
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn claim_activity_level_reward(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, ActivityLevelRewardError> {
        let config = tables
            .activity_level_rewards
            .get(id)
            .ok_or(ActivityLevelRewardError::Unknown(id))?;
        if self.level < config.required_level {
            return Err(ActivityLevelRewardError::LevelLocked {
                id,
                required: config.required_level,
            });
        }
        if self.activity_level_rewards.contains(&id) {
            return Err(ActivityLevelRewardError::AlreadyClaimed(id));
        }
        let rewards = config.reward.clone();
        self.activity_level_rewards.push(id);
        self.activity_level_rewards.sort_unstable();
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn open_activities(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataActivityOpenInfo> {
        tables
            .activities
            .rows
            .iter()
            .filter_map(|activity| {
                let remain_time = activity_seconds_remaining(activity, now, zone_offset)?;
                let in_status = activity.limit_type.iter().all(|limit| {
                    self.condition_progress(limit.key, &limit.value, tables, now)
                        >= condition_total(limit.key, &limit.value)
                });
                Some(DcNetDataActivityOpenInfo {
                    aid: activity.id,
                    remain_time,
                    in_status,
                })
            })
            .collect()
    }

    pub fn check_in_status(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> (Vec<DcNetDataCheckInfo>, bool) {
        let day = common::time::daily_refresh_day(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        let active: Vec<_> = self
            .open_activities(tables, now, zone_offset)
            .into_iter()
            .filter_map(|info| {
                tables
                    .activities
                    .get(info.aid)
                    .filter(|activity| activity.activity_type == 1 && info.in_status)
                    .map(|activity| activity.id)
            })
            .collect();
        let mut changed = false;
        for aid in &active {
            let max_days = tables
                .check_ins_by_activity
                .get(*aid)
                .into_iter()
                .flatten()
                .map(|checkin| checkin.num)
                .max()
                .unwrap_or_default();
            if let Some(state) = self.checkins.iter_mut().find(|state| state.aid == *aid) {
                if day > state.last_check_day {
                    state.check_days = state.check_days.saturating_add(1).min(max_days);
                    state.last_check_day = day;
                    changed = true;
                }
            } else {
                self.checkins.push(CheckinState {
                    aid: *aid,
                    check_days: i32::from(max_days > 0),
                    last_check_day: day,
                    claimed_days: Vec::new(),
                });
                changed = true;
            }
        }
        let infos = active
            .into_iter()
            .filter_map(|aid| {
                let state = self.checkins.iter().find(|state| state.aid == aid)?;
                Some(DcNetDataCheckInfo {
                    check_days: state.check_days,
                    has_get_days: state.claimed_days.clone(),
                    cur_aid: aid,
                })
            })
            .collect();
        (infos, changed)
    }

    pub fn claim_check_in_reward(
        &mut self,
        aid: i32,
        get_day: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataTakeRewardRes, CheckinError> {
        let activity = tables
            .activities
            .get(aid)
            .filter(|activity| activity.activity_type == 1)
            .ok_or(CheckinError::UnknownActivity(aid))?;
        if activity_seconds_remaining(activity, now, zone_offset).is_none() {
            return Err(CheckinError::Inactive(aid));
        }
        self.check_in_status(tables, now, zone_offset);
        let reward = tables
            .check_ins_by_activity
            .get(aid)
            .into_iter()
            .flatten()
            .find(|checkin| checkin.num == get_day)
            .map(|checkin| checkin.reward.clone())
            .ok_or(CheckinError::UnknownDay { aid, day: get_day })?;
        let state = self
            .checkins
            .iter_mut()
            .find(|state| state.aid == aid)
            .ok_or(CheckinError::Inactive(aid))?;
        if get_day <= 0 || get_day > state.check_days {
            return Err(CheckinError::Unavailable { aid, day: get_day });
        }
        if state.claimed_days.contains(&get_day) {
            return Err(CheckinError::AlreadyClaimed { aid, day: get_day });
        }
        state.claimed_days.push(get_day);
        state.claimed_days.sort_unstable();
        let mut result = DcNetDataTakeRewardRes::default();
        self.add_reward(reward.key, reward.value, tables, now, &mut result);
        Ok(result)
    }

    pub fn seven_day_activity_status(
        &self,
        tables: &GameTables,
        now: i32,
    ) -> (Vec<DcNetDataAct7Day>, Vec<i32>) {
        let list = tables
            .seven_day_activities
            .rows
            .iter()
            .filter_map(|activity| self.seven_day_activity_entry(activity.id, tables, now))
            .collect();
        (list, self.activity_7day_point_claims.clone())
    }

    pub(super) fn completed_seven_day_activities_since(
        &self,
        before: &[DcNetDataAct7Day],
        tables: &GameTables,
        now: i32,
    ) -> Vec<DcNetDataAct7Day> {
        self.seven_day_activity_status(tables, now)
            .0
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

    pub fn claim_seven_day_activity_reward(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataAct7Day, DcNetDataTakeRewardRes), Activity7DayError> {
        let config = tables
            .seven_day_activities
            .get(id)
            .ok_or(Activity7DayError::Unknown(id))?;
        let rewards = config.reward.clone();
        let mut info = self
            .seven_day_activity_entry(id, tables, now)
            .ok_or(Activity7DayError::Locked(id))?;
        if info.took_at > 0 {
            return Err(Activity7DayError::AlreadyClaimed(id));
        }
        if info.progress < info.total {
            return Err(Activity7DayError::Incomplete(id));
        }

        info.took_at = i64::from(now);
        self.activity_7day_claims.push(info);
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok((info, result))
    }

    pub fn claim_seven_day_activity_milestone(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, Activity7DayError> {
        let config = tables
            .seven_day_activity_milestones
            .get(id)
            .ok_or(Activity7DayError::Unknown(id))?;
        if self.activity_7day_point_claims.contains(&id) {
            return Err(Activity7DayError::AlreadyClaimed(id));
        }
        let limit = config
            .limit_type
            .first()
            .filter(|limit| limit.key == 50)
            .ok_or(Activity7DayError::InvalidPoint(id))?;
        let mut values = limit
            .value
            .split('|')
            .filter_map(|value| value.parse::<i32>().ok());
        let (Some(item_id), Some(required)) = (values.next(), values.next()) else {
            return Err(Activity7DayError::InvalidPoint(id));
        };
        let amount = self
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .map_or(0, |item| item.amount);
        if amount < required {
            return Err(Activity7DayError::Incomplete(id));
        }

        let rewards = config.reward.clone();
        self.activity_7day_point_claims.push(id);
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    fn seven_day_activity_entry(
        &self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Option<DcNetDataAct7Day> {
        let config = tables.seven_day_activities.get(id)?;
        if config.group > self.account_days(now).min(7) {
            return None;
        }
        let limit = config.finish_limit.first()?;
        let total = if limit.key == 81 {
            1
        } else {
            limit.value.rsplit('|').next()?.parse().ok()?
        };
        let took_at = self
            .activity_7day_claims
            .iter()
            .find(|claim| claim.id == id)
            .map_or(0, |claim| claim.took_at);
        Some(DcNetDataAct7Day {
            id,
            took_at,
            progress: self.condition_progress(limit.key, &limit.value, tables, now),
            total,
        })
    }

    fn account_days(&self, now: i32) -> i32 {
        (common::time::day_index(now) - common::time::day_index(self.created_at) + 1).max(1)
    }

    pub(super) fn condition_progress(
        &self,
        condition: i32,
        value: &str,
        tables: &GameTables,
        now: i32,
    ) -> i32 {
        match condition {
            3 | 4 => self.account_days(now),
            5 => self.level,
            6 => self.world_level(tables),
            13 => value
                .split('|')
                .next()
                .and_then(|level| level.parse::<i32>().ok())
                .map_or(0, |level| {
                    self.roles
                        .iter()
                        .filter_map(|role| role.role_basic_info.as_ref())
                        .filter(|role| role.level >= level)
                        .count() as i32
                }),
            16 | 61 => self.gachas.iter().map(|gacha| gacha.all_count).sum(),
            17 => self.roles.len() as i32,
            25 => value
                .rsplit('|')
                .next()
                .and_then(|level| level.parse::<i32>().ok())
                .map_or(0, |level| {
                    self.equips
                        .iter()
                        .filter(|equipment| equipment.level >= level)
                        .count() as i32
                }),
            26 => value
                .rsplit('|')
                .next()
                .and_then(|level| level.parse::<i32>().ok())
                .map_or(0, |level| {
                    self.partners
                        .iter()
                        .filter(|partner| partner.lv >= level)
                        .count() as i32
                }),
            28 => value
                .split('|')
                .next()
                .and_then(|quality| quality.parse::<i32>().ok())
                .map_or(0, |quality| {
                    self.roles
                        .iter()
                        .filter_map(|role| role.role_basic_info.as_ref())
                        .filter(|role| role.qua >= quality)
                        .count() as i32
                }),
            29 => self.partners.len() as i32,
            41 => value
                .split('|')
                .next()
                .and_then(|dungeon_type| dungeon_type.parse::<i32>().ok())
                .and_then(|dungeon_type| self.dungeon_clears.get(&dungeon_type).copied())
                .unwrap_or_default(),
            50 => value
                .split('|')
                .next()
                .and_then(|id| id.parse::<i32>().ok())
                .and_then(|id| self.items.iter().find(|item| item.item_id == id))
                .map_or(0, |item| item.amount),
            51 => value
                .split('|')
                .next()
                .and_then(|id| id.parse::<i32>().ok())
                .and_then(|id| self.item_spent.get(&id).copied())
                .unwrap_or_default(),
            81 => value
                .split('|')
                .filter_map(|id| id.parse::<i32>().ok())
                .all(|id| {
                    self.tasks
                        .iter()
                        .any(|task| task.id == id && task.status == TaskStatus::Done as i32)
                })
                .into(),
            85 => self
                .user_guides
                .iter()
                .chain(&self.city_guides)
                .any(|id| id == value)
                .into(),
            91 => self
                .reward_boxes
                .iter()
                .filter(|reward_box| reward_box.status == 1)
                .count() as i32,
            102 => self
                .roles
                .iter()
                .filter_map(|role| role.role_basic_info.as_ref())
                .map(|role| role.level)
                .max()
                .unwrap_or_default(),
            103 => self
                .partners
                .iter()
                .map(|partner| partner.lv)
                .max()
                .unwrap_or_default(),
            104 => self
                .equips
                .iter()
                .map(|equipment| equipment.level)
                .max()
                .unwrap_or_default(),
            201 => value
                .split('|')
                .next()
                .and_then(|quality| quality.parse::<i32>().ok())
                .map_or(0, |quality| {
                    self.partners
                        .iter()
                        .filter(|partner| partner.quality >= quality)
                        .count() as i32
                }),
            203 => value
                .split('|')
                .next()
                .and_then(|quality| quality.parse::<i32>().ok())
                .map_or(0, |quality| {
                    self.equips
                        .iter()
                        .filter(|equipment| equipment.quality >= quality)
                        .count() as i32
                }),
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests;
