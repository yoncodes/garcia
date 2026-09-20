use std::collections::BTreeMap;

use super::*;

impl Player {
    pub fn current_traced_task_ids(&self, tables: &GameTables) -> Vec<i32> {
        let current_sort = self
            .tasks
            .iter()
            .filter(|state| state.status == TaskStatus::Picked as i32)
            .filter_map(|state| tables.tasks.get(state.id))
            .filter(|task| task.task_group == self.traced_task_group && task.sort != 0)
            .map(|task| task.sort)
            .min();
        self.tasks
            .iter()
            .filter(|state| state.status == TaskStatus::Picked as i32)
            .filter_map(|state| tables.tasks.get(state.id))
            .filter(|task| {
                task.task_group == self.traced_task_group && Some(task.sort) == current_sort
            })
            .map(|task| task.id)
            .collect()
    }

    pub(super) fn store_task_status(&mut self, task: DcNetDataTaskStatus) -> DcNetDataTaskStatus {
        if let Some(current) = self.tasks.iter_mut().find(|current| current.id == task.id) {
            *current = task;
        } else {
            self.tasks.push(task);
        }
        task
    }

    pub(crate) fn activate_task(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTaskStatus, TaskCompleteError> {
        tables.tasks.get(id).ok_or(TaskCompleteError::Unknown(id))?;
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            if task.status != TaskStatus::Done as i32 {
                task.status = TaskStatus::Picked as i32;
                task.picked_at = i64::from(now);
            }
            return Ok(*task);
        }
        let task = DcNetDataTaskStatus {
            id,
            status: TaskStatus::Picked as i32,
            picked_at: i64::from(now),
            ..Default::default()
        };
        Ok(self.store_task_status(task))
    }

    pub fn daily_login_updates(&self, tables: &GameTables) -> Vec<DcNetDataTaskCycle> {
        self.daily_tasks
            .iter()
            .filter(|task| {
                tables
                    .daily_tasks
                    .get(task.id)
                    .and_then(|definition| definition.finish_limit.first())
                    .is_some_and(|limit| limit.key == 4)
            })
            .copied()
            .collect()
    }

    pub fn set_traced_task_group(
        &mut self,
        group_id: i32,
        tables: &GameTables,
    ) -> Result<i32, TaskGoalError> {
        if group_id != 0
            && !self.tasks.iter().any(|task| {
                task.status == TaskStatus::Picked as i32
                    && tables
                        .tasks
                        .get(task.id)
                        .is_some_and(|config| config.task_group == group_id)
            })
        {
            return Err(TaskGoalError::Inactive(group_id));
        }
        self.traced_task_group = group_id;
        Ok(group_id)
    }

    pub fn advance_port_tasks(
        &mut self,
        gameplay_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<TaskProgressOutcome>, TaskCompleteError> {
        let Some(port) = self.ports.iter().find(|port| port.id == gameplay_id) else {
            return Ok(Vec::new());
        };
        let (pass_count, all_count) = (port.pass_cnt, port.all_cnt);
        let candidates = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Picked as i32)
            .filter_map(|task| {
                let condition = tables.tasks.get(task.id)?.done_key.first()?;
                let (target, total) = condition.value.split_once('|')?;
                let target = target.parse::<i32>().ok()?;
                let total = total.parse::<i32>().ok()?;
                matches!(condition.key, 39 | 40).then_some((task.id, condition.key, target, total))
            })
            .filter(|(_, _, target, _)| *target == gameplay_id)
            .collect::<Vec<_>>();

        self.advance_tasks(
            candidates
                .into_iter()
                .map(|(task_id, condition, _, total)| {
                    let progress = if condition == 39 {
                        pass_count
                    } else {
                        all_count
                    };
                    (task_id, progress, total)
                })
                .collect(),
            tables,
            now,
        )
    }

    pub fn advance_interact_tasks(
        &mut self,
        object_id: &str,
        status: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<TaskProgressOutcome>, TaskCompleteError> {
        let candidates = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Picked as i32)
            .filter_map(|task| {
                let condition = tables.tasks.get(task.id)?.done_key.first()?;
                let (target, expected) = condition.value.rsplit_once('|')?;
                let expected = expected.parse::<i32>().ok()?;
                (condition.key == 83 && target == object_id && status == expected)
                    .then_some((task.id, 1, 1))
            })
            .collect();
        self.advance_tasks(candidates, tables, now)
    }

    pub fn advance_level_tasks(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<TaskProgressOutcome>, TaskCompleteError> {
        let candidates = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Picked as i32)
            .filter_map(|task| {
                let condition = tables.tasks.get(task.id)?.done_key.first()?;
                let total = condition.value.parse::<i32>().ok()?;
                (condition.key == 5).then_some((task.id, self.level, total))
            })
            .collect();
        self.advance_tasks(candidates, tables, now)
    }

    pub fn advance_dungeon_tasks(
        &mut self,
        gameplay_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<TaskProgressOutcome>, TaskCompleteError> {
        let Some(dungeon_id) = tables
            .dungeon_ports
            .get(gameplay_id)
            .and_then(|port| tables.dungeon_port_groups.get(port.port_group_id))
            .map(|group| group.dungeon_id)
        else {
            return Ok(Vec::new());
        };
        let completed = self
            .completed_dungeons
            .iter()
            .filter(|gameplay_id| {
                tables
                    .dungeon_ports
                    .get(**gameplay_id)
                    .and_then(|port| tables.dungeon_port_groups.get(port.port_group_id))
                    .is_some_and(|group| group.dungeon_id == dungeon_id)
            })
            .count() as i32;
        let candidates = self
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Picked as i32)
            .filter_map(|task| {
                let condition = tables.tasks.get(task.id)?.done_key.first()?;
                let (target, total) = condition.value.split_once('|')?;
                let target = target.parse::<i32>().ok()?;
                let total = total.parse::<i32>().ok()?;
                (condition.key == 43 && target == dungeon_id).then_some((task.id, completed, total))
            })
            .collect();
        self.advance_tasks(candidates, tables, now)
    }

    fn advance_tasks(
        &mut self,
        candidates: Vec<(i32, i32, i32)>,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<TaskProgressOutcome>, TaskCompleteError> {
        let mut outcomes = Vec::with_capacity(candidates.len());
        for (task_id, progress, total) in candidates {
            let task = self
                .tasks
                .iter_mut()
                .find(|task| task.id == task_id)
                .unwrap();
            task.progress = progress.min(total);
            task.total = total;
            let progress = *task;
            let completion = (progress.progress >= progress.total)
                .then(|| self.complete_task(task_id, 0, tables, now))
                .transpose()?;
            outcomes.push(TaskProgressOutcome {
                progress,
                completion,
            });
        }
        Ok(outcomes)
    }

    pub fn claim_mission(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataMissonInfo, DcNetDataTakeRewardRes), MissionClaimError> {
        let rewards = tables
            .missions
            .get(id)
            .ok_or(MissionClaimError::Unknown(id))?
            .rewards
            .clone();
        let index = self
            .missions
            .iter()
            .position(|mission| mission.misson_id == id)
            .ok_or(MissionClaimError::Inactive(id))?;
        let mission = &self.missions[index];
        if mission.taken {
            return Err(MissionClaimError::AlreadyClaimed(id));
        }
        if mission.curr_num < mission.total_num {
            return Err(MissionClaimError::Incomplete(id));
        }

        self.missions[index].taken = true;
        self.missions[index].take_time = now;
        let mission = self.missions[index];
        let items = rewards
            .into_iter()
            .filter_map(|reward| self.add_item(reward.key, reward.value, tables))
            .collect();
        Ok((
            mission,
            DcNetDataTakeRewardRes {
                items,
                ..Default::default()
            },
        ))
    }

    pub fn advance_level_missions(&mut self, now: i32) -> Vec<DcNetDataMissonInfo> {
        let mut updates = self
            .missions
            .iter_mut()
            .filter(|mission| {
                !mission.taken
                    && mission.curr_num < mission.total_num
                    && self.level >= mission.total_num
            })
            .map(|mission| {
                mission.curr_num = self.level;
                mission.created_at = now;
                *mission
            })
            .collect::<Vec<_>>();
        updates.sort_by(|left, right| right.total_num.cmp(&left.total_num));
        updates
    }

    pub fn advance_features(&mut self, tables: &GameTables) -> Vec<DcNetDataFeat> {
        let mut updates = Vec::new();
        for config in &tables.feature_unlocks.rows {
            if config.enabled == 0 || config.conditions.is_empty() {
                continue;
            }
            let unlocked = config.conditions.iter().all(|limit| match limit.key {
                81 => limit.value.split('|').all(|value| {
                    value.parse::<i32>().ok().is_some_and(|id| {
                        self.tasks
                            .iter()
                            .any(|task| task.id == id && task.status == TaskStatus::Done as i32)
                    })
                }),
                84 => limit
                    .value
                    .parse::<i32>()
                    .ok()
                    .is_some_and(|id| self.tasks.iter().any(|task| task.id == id)),
                _ => false,
            });
            if unlocked {
                if let Some(feat) = self.feats.iter_mut().find(|feat| feat.id == config.id) {
                    if feat.status != FeatStatus::FeatUnlocked as i32 {
                        feat.status = FeatStatus::FeatUnlocked as i32;
                        updates.push(*feat);
                    }
                } else {
                    let feat = DcNetDataFeat {
                        id: config.id,
                        status: FeatStatus::FeatUnlocked as i32,
                    };
                    self.feats.push(feat);
                    updates.push(feat);
                }
            }
        }
        updates
    }

    pub fn unlock_feature(
        &mut self,
        feat_id: i32,
        tables: &GameTables,
    ) -> Result<DcNetDataFeat, FeatureError> {
        self.advance_features(tables);
        if !self.feats.iter().any(|feat| feat.id == feat_id) {
            let config = tables
                .feature_unlocks
                .get(feat_id)
                .filter(|config| config.enabled != 0 && config.conditions.is_empty())
                .ok_or(FeatureError::NotUnlockable(feat_id))?;
            self.feats.push(DcNetDataFeat {
                id: config.id,
                status: FeatStatus::FeatUnlocked as i32,
            });
        }
        let feat = self
            .feats
            .iter_mut()
            .find(|feat| feat.id == feat_id)
            .ok_or(FeatureError::NotUnlockable(feat_id))?;
        if feat.status == FeatStatus::FeatUnlockable as i32 {
            feat.status = FeatStatus::FeatUnlocked as i32;
        }
        if feat.status != FeatStatus::FeatUnlocked as i32 {
            return Err(FeatureError::NotUnlockable(feat_id));
        }
        Ok(*feat)
    }

    pub fn unlock_all_features(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<FeatureUnlockOutcome, FeatureError> {
        let mut required_tasks: BTreeMap<i32, i32> = BTreeMap::new();
        for feature in tables
            .feature_unlocks
            .rows
            .iter()
            .filter(|feature| feature.enabled != 0)
        {
            for requirement in &feature.conditions {
                let status = match requirement.key {
                    81 => TaskStatus::Done as i32,
                    84 => TaskStatus::Picked as i32,
                    condition => {
                        return Err(FeatureError::UnsupportedRequirement {
                            feature_id: feature.id,
                            condition,
                        });
                    }
                };
                for value in requirement.value.split('|') {
                    let task_id = value.parse::<i32>().map_err(|_| {
                        FeatureError::InvalidRequirementValue {
                            feature_id: feature.id,
                            value: requirement.value.clone(),
                        }
                    })?;
                    if tables.tasks.get(task_id).is_none() {
                        return Err(FeatureError::UnknownRequirementTask(task_id));
                    }
                    required_tasks
                        .entry(task_id)
                        .and_modify(|current| *current = (*current).max(status))
                        .or_insert(status);
                }
            }
        }

        let mut requirements = Vec::new();
        let mut completions = Vec::new();
        let mut ports = Vec::new();
        let mut level_up_data = None;
        let mut features = Vec::new();
        for (task_id, status) in required_tasks {
            let outcome = self
                .advance_task_chain(task_id, status, tables, now)
                .map_err(|error| FeatureError::Progression(error.to_string()))?;
            requirements.extend(outcome.tasks);
            completions.extend(outcome.completions);
            ports.extend(outcome.ports);
            features.extend(outcome.features);
            if outcome.level_update.is_some() {
                level_up_data = outcome.level_update;
            }
        }
        requirements.sort_unstable_by_key(|task| task.id);
        requirements.dedup_by_key(|task| task.id);
        ports.sort_unstable_by_key(|port| port.id);
        ports.dedup_by_key(|port| port.id);
        features.extend(self.advance_features(tables));
        for config in tables
            .feature_unlocks
            .rows
            .iter()
            .filter(|config| config.enabled != 0)
        {
            let already_unlocked = self
                .feats
                .iter()
                .find(|feature| feature.id == config.id)
                .is_some_and(|feature| feature.status == FeatStatus::FeatUnlocked as i32);
            let feature = self.unlock_feature(config.id, tables)?;
            if !already_unlocked {
                features.push(feature);
            }
        }
        features.sort_unstable_by_key(|feature| feature.id);
        features.dedup_by_key(|feature| feature.id);
        Ok(FeatureUnlockOutcome {
            requirements,
            completions,
            ports,
            level_up_data,
            features,
        })
    }

    pub fn complete_task(
        &mut self,
        id: i32,
        select: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<TaskCompleteOutcome, TaskCompleteError> {
        let old_world_level = self.world_level(tables);
        let previous_achievements = self.achievements(tables, now);
        let config = tables
            .tasks
            .get(id)
            .ok_or(TaskCompleteError::Unknown(id))?
            .clone();
        let region = tables
            .region_tasks_by_region
            .rows
            .iter()
            .find(|row| row.task_group_id == config.task_group)
            .and_then(|row| tables.city_regions.get(row.region_id));
        if let Some(reward) = config.rewards.iter().find(|reward| {
            let known = tables.items.get(reward.key).is_some()
                || tables.partners.get(reward.key).is_some()
                || tables.equipment.get(reward.key).is_some()
                || tables.collections.get(reward.key).is_some()
                || tables.maid_skins.get(reward.key).is_some();
            !known
                && !region.is_some_and(|region| {
                    reward.key == region.reputation_exp
                        || (region.reputation_money != 0
                            && tables.items.get(region.reputation_money).is_some())
                })
        }) {
            return Err(TaskCompleteError::UnsupportedReward {
                task_id: id,
                reward_id: reward.key,
            });
        }
        let index = self
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or(TaskCompleteError::Inactive(id))?;
        if self.tasks[index].status == TaskStatus::Done as i32 {
            return Err(TaskCompleteError::AlreadyComplete(id));
        }
        let previous_archives = self.unlocked_archive_ids(tables, now);

        self.tasks[index].status = TaskStatus::Done as i32;
        let current = self.tasks[index];
        let next_ids = config
            .next
            .get(if select > 0 { (select - 1) as usize } else { 0 })
            .into_iter()
            .flat_map(|entry| entry.split('|'))
            .filter_map(|id| id.parse::<i32>().ok())
            .collect::<Vec<_>>();
        let next = next_ids
            .into_iter()
            .map(|id| self.activate_task(id, tables, now))
            .collect::<Result<Vec<_>, _>>()?;
        let mut rewards = DcNetDataTakeRewardRes::default();
        let mut level_up_data = None;
        let mut region_reward = None;
        for reward in config.rewards {
            if reward.key == PLAYER_EXP_ITEM_ID {
                let update = self.add_user_exp(reward.value, tables);
                rewards.items.extend(update.items.iter().copied());
                level_up_data = Some(update);
            } else if let Some(region) = region.filter(|region| reward.key == region.reputation_exp)
            {
                let item = self
                    .add_region_exp(region.id, reward.value, tables)
                    .map_err(|_| TaskCompleteError::UnsupportedReward {
                        task_id: id,
                        reward_id: reward.key,
                    })?;
                region_reward = Some((region.id, item));
            } else {
                let reward_id = if tables.items.get(reward.key).is_none() {
                    region.map_or(reward.key, |region| region.reputation_money)
                } else {
                    reward.key
                };
                self.add_reward(reward_id, reward.value, tables, now, &mut rewards);
            }
        }
        let new_world_level = self.world_level(tables);
        let world_level = (new_world_level > old_world_level).then_some(new_world_level);
        let achievement_updates =
            self.completed_achievements_since(&previous_achievements, tables, now);
        let archive_updates = self.newly_unlocked_archives_since(&previous_archives, tables, now);
        let mission_updates = self.advance_level_missions(now);
        let feat_updates = self.advance_features(tables);
        let album_updates = self.unlock_available_albums(tables, now);
        let sms_updates = self.sync_sms(tables);
        let interact_updates = self.refresh_interactions(tables);
        Ok(TaskCompleteOutcome {
            current,
            changed: next,
            achievement_updates,
            archive_updates,
            album_updates,
            sms_updates,
            interact_updates,
            rewards,
            level_up_data,
            region_reward,
            world_level,
            mission_updates,
            feat_updates,
        })
    }

    pub(super) fn add_reward(
        &mut self,
        id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
        rewards: &mut DcNetDataTakeRewardRes,
    ) {
        if let Some(item) = self.add_item(id, amount, tables) {
            rewards.items.push(item);
            return;
        }
        if tables.profile_avatars.get(id).is_some() {
            let profile = DcNetDataProfileAvatar { id };
            if !self
                .profile_avatars
                .iter()
                .any(|owned| owned.id == profile.id)
            {
                self.profile_avatars.push(profile);
                rewards.profileavatar_rewards.push(profile);
            }
            return;
        }
        if tables.profile_frames.get(id).is_some() {
            let profile = DcNetDataProfileFrame { id, end_time: 0 };
            if !self
                .profile_frames
                .iter()
                .any(|owned| owned.id == profile.id)
            {
                self.profile_frames.push(profile);
                rewards.profileframe_rewards.push(profile);
            }
            return;
        }
        if tables.profile_cards.get(id).is_some() {
            let profile = DcNetDataProfileCard { id };
            if !self
                .profile_cards
                .iter()
                .any(|owned| owned.id == profile.id)
            {
                self.profile_cards.push(profile);
                rewards.profilecard_rewards.push(profile);
            }
            return;
        }
        if tables.profile_titles.get(id).is_some() {
            let profile = DcNetDataProfileTitle {
                id,
                end_time: 0,
                num: 1,
            };
            if !self
                .profile_titles
                .iter()
                .any(|owned| owned.id == profile.id)
            {
                self.profile_titles.push(profile);
            }
            rewards.profiletitle_rewards.push(profile);
            return;
        }
        if tables.maids.get(id).is_some() {
            for _ in 0..amount.max(0) {
                if let Some(role) = self
                    .roles
                    .iter()
                    .find(|role| {
                        role.role_basic_info
                            .as_ref()
                            .is_some_and(|info| info.game_role_id == id)
                    })
                    .cloned()
                {
                    let conversion = self
                        .role_duplicate_reward(id, None, tables)
                        .expect("owned maids have a configured duplicate conversion");
                    let duplicate = self.add_item(conversion.key, conversion.value, tables);
                    rewards.role_rewards.push(DcNetDataRoleReward {
                        // The client reads role_info before checking
                        // on_duplicate, including for an owned maid.
                        role_info: Some(role),
                        on_duplicate: duplicate,
                        reward: None,
                    });
                } else if let Some(role) = gacha_role(self.uid, id, tables) {
                    self.roles.push(role.clone());
                    rewards.role_rewards.push(DcNetDataRoleReward {
                        role_info: Some(role),
                        on_duplicate: None,
                        reward: None,
                    });
                    if let Some(avatar) = self.unlock_role_avatar(id, tables) {
                        rewards.profileavatar_rewards.push(avatar);
                    }
                }
            }
            self.record_current_archive_unlocks(tables, now);
            return;
        }
        if let Some(config) = tables.partners.get(id) {
            for _ in 0..amount.max(0) {
                let partner = DcNetDataPartner {
                    id: self.next_partner_instance_id(id),
                    lv: 1,
                    partner_id: id,
                    quality: config.quality,
                    reson_lv: 1,
                    skill_lv: 1,
                    creat_at: i64::from(now),
                    ..Default::default()
                };
                self.partners.push(partner);
                rewards.partner_rewards.push(DcNetDataPartnerReward {
                    partner: Some(partner),
                    reward: None,
                });
            }
            self.record_current_archive_unlocks(tables, now);
            return;
        }
        if tables.equipment.get(id).is_some() {
            rewards.equips.extend(
                self.generate_relics(id, amount, tables, now)
                    .expect("equipment definition was checked above"),
            );
            return;
        }
        if tables.team_equipment.get(id).is_some() {
            for _ in 0..amount.max(0) {
                if let Ok(equip) = self.grant_team_equip(id, tables) {
                    rewards.teamequip_rewards.push(equip);
                }
            }
            return;
        }
        if tables.skill_stones.get(id).is_some() {
            for _ in 0..amount.max(0) {
                if let Ok(stone) = self.grant_skillstone(id, tables) {
                    rewards.stone_rewards.push(stone);
                }
            }
            return;
        }
        if tables.team_cores.get(id).is_some() {
            for _ in 0..amount.max(0) {
                if let Ok(core) = self.grant_team_core(id, tables) {
                    rewards.teamecore_rewards.push(core);
                }
            }
            return;
        }
        if tables.collections.get(id).is_some() && amount > 0 {
            let collection = DcNetDataCollection {
                cid: id,
                in_time: i64::from(now),
                reward: false,
            };
            self.collections.push(collection);
            rewards.collection_rewards.push(collection);
            return;
        }
        if tables.maid_skins.get(id).is_some() && amount > 0 && !self.skins.contains(&id) {
            self.skins.push(id);
            rewards.skin_rewards.push(id);
        }
    }

    pub(super) fn add_item(
        &mut self,
        id: i32,
        amount: i32,
        tables: &GameTables,
    ) -> Option<DcNetDataItem> {
        if let Some(item) = self.items.iter_mut().find(|item| item.item_id == id) {
            item.amount = item.amount.saturating_add(amount);
            if amount > 0 {
                let acquired = self.item_acquired.entry(id).or_default();
                *acquired = acquired.saturating_add(amount);
            }
            return Some(*item);
        }
        let config = tables.items.get(id)?;
        let item = DcNetDataItem {
            user_item_id: make_entity_id(self.uid, id),
            item_id: id,
            amount,
            remain_sec: 0,
            quality: config.quality,
        };
        if amount > 0 {
            let acquired = self.item_acquired.entry(id).or_default();
            *acquired = acquired.saturating_add(amount);
        }
        self.items.push(item);
        Some(item)
    }

    pub(super) fn add_user_exp(&mut self, amount: i32, tables: &GameTables) -> DcNetDataAttrUpData {
        let old_level = self.level;
        let world_level = self.world_level(tables);
        let index = self
            .items
            .iter()
            .position(|item| item.item_id == PLAYER_EXP_ITEM_ID)
            .unwrap_or_else(|| {
                let quality = tables
                    .items
                    .get(PLAYER_EXP_ITEM_ID)
                    .map_or(0, |item| item.quality);
                self.items.push(DcNetDataItem {
                    user_item_id: make_entity_id(self.uid, PLAYER_EXP_ITEM_ID),
                    item_id: PLAYER_EXP_ITEM_ID,
                    quality,
                    ..Default::default()
                });
                self.items.len() - 1
            });
        let mut exp = self.items[index].amount.saturating_add(amount);
        while let Some(level) = tables.player_levels.get(self.level) {
            if level.exp <= 0 || exp < level.exp || world_level < level.required_world_level {
                break;
            }
            exp -= level.exp;
            self.level += 1;
        }
        self.items[index].amount = exp;
        DcNetDataAttrUpData {
            attr_lv: self.level,
            attr_exp: 0,
            add_exp: amount,
            items: vec![self.items[index]],
            is_lv_up: self.level > old_level,
        }
    }

    pub(super) fn advance_daily_condition(
        &mut self,
        condition: i32,
        amount: i32,
        tables: &GameTables,
    ) -> Vec<DcNetDataTaskCycle> {
        let mut updates = Vec::new();
        for task in &mut self.daily_tasks {
            let matches = tables
                .daily_tasks
                .get(task.id)
                .and_then(|config| config.finish_limit.first())
                .is_some_and(|limit| limit.key == condition);
            if matches && !task.taken {
                let progress = task.progress.saturating_add(amount).min(task.total);
                if progress != task.progress {
                    task.progress = progress;
                    updates.push(*task);
                }
            }
        }
        updates
    }

    pub(super) fn advance_dungeon_daily_tasks(
        &mut self,
        dungeon_type: i32,
        tables: &GameTables,
    ) -> Vec<DcNetDataTaskCycle> {
        let mut updates = Vec::new();
        for task in &mut self.daily_tasks {
            let Some(limit) = tables
                .daily_tasks
                .get(task.id)
                .and_then(|config| config.finish_limit.first())
            else {
                continue;
            };
            let matches = limit.key == 32
                || (limit.key == 41
                    && limit
                        .value
                        .split('|')
                        .next()
                        .and_then(|value| value.parse::<i32>().ok())
                        == Some(dungeon_type));
            if matches && !task.taken {
                let progress = task.progress.saturating_add(1).min(task.total);
                if progress != task.progress {
                    task.progress = progress;
                    if task.progress >= task.total {
                        updates.push(*task);
                    }
                }
            }
        }
        updates
    }
}
#[cfg(test)]
mod tests;
