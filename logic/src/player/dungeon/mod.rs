use super::*;

// Made up drop rates for gear fuck the devs
const RELIC_QUALITY_WEIGHTS: [(i32, u32); 3] = [(4, 70), (5, 25), (6, 5)];

impl Player {
    pub fn start_dungeon(
        &mut self,
        id: i32,
        dungeon_type: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(), DungeonError> {
        let _ = self.refresh_heat(tables, now);
        let config = self.require_dungeon_config(id, dungeon_type, tables, now)?;
        self.validate_dungeon_cost(config, 1)?;
        self.gameplay_id = id;
        Ok(())
    }

    pub fn settle_dungeon(
        &mut self,
        settlement: &DcNetDataParamsSettlement,
        multiplier: i32,
        dungeon_type: i32,
        tables: &GameTables,
        runtime: DungeonRuntime,
    ) -> Result<DungeonSettlementOutcome, DungeonError> {
        if self.gameplay_id != settlement.id {
            return Err(DungeonError::NotActive {
                expected: self.gameplay_id,
                actual: settlement.id,
            });
        }
        let _ = self.refresh_heat(tables, runtime.now);
        let config = self
            .require_dungeon_config(settlement.id, dungeon_type, tables, runtime.now)?
            .clone();
        self.validate_multiplier(multiplier, config.n_times)?;

        let previous_achievements = self.achievements(tables, runtime.now);
        let previous_archives = self.unlocked_archive_ids(tables, runtime.now);
        let previous_seven_day = self.seven_day_activity_status(tables, runtime.now).0;
        let previous_battle_pass =
            self.battle_pass_task_snapshot(tables, runtime.now, runtime.zone_offset);

        let (rewards, remains, heat_update, user_up_data, mission_updates, daily_updates) =
            if settlement.is_completed == 0 {
                (
                    DcNetDataTakeRewardRes::default(),
                    Vec::new(),
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                )
            } else {
                self.validate_dungeon_cost(&config, multiplier)?;
                let (remains, heat_update) =
                    self.consume_dungeon_cost(&config, multiplier, tables, runtime.now);
                let rewards = self.grant_dungeon_rewards(&config, multiplier, tables, runtime.now);
                let player_exp =
                    dungeon_player_exp(&config, multiplier, tables, runtime.player_exp_per_stamina);
                let user_up_data = (player_exp > 0).then(|| self.add_user_exp(player_exp, tables));
                if !self.completed_dungeons.contains(&settlement.id) {
                    self.completed_dungeons.push(settlement.id);
                }
                *self.dungeon_clears.entry(dungeon_type).or_default() += 1;
                let mission_updates = if user_up_data.as_ref().is_some_and(|update| update.is_lv_up)
                {
                    self.advance_level_missions(runtime.now)
                } else {
                    Vec::new()
                };
                let daily_updates = self.advance_dungeon_daily_tasks(dungeon_type, tables);
                (
                    rewards,
                    remains,
                    heat_update,
                    user_up_data,
                    mission_updates,
                    daily_updates,
                )
            };
        let achievement_updates =
            self.completed_achievements_since(&previous_achievements, tables, runtime.now);
        let archive_updates =
            self.newly_unlocked_archives_since(&previous_archives, tables, runtime.now);
        let seven_day_updates =
            self.completed_seven_day_activities_since(&previous_seven_day, tables, runtime.now);
        let battle_pass_updates = self.changed_battle_pass_tasks_since(
            &previous_battle_pass,
            tables,
            runtime.now,
            runtime.zone_offset,
        );

        Ok(DungeonSettlementOutcome {
            gameplay_id: settlement.id,
            rewards,
            remains,
            heat_update,
            user_up_data,
            mission_updates,
            achievement_updates,
            archive_updates,
            daily_updates,
            seven_day_updates,
            battle_pass_updates,
        })
    }

    pub fn sweep_dungeon(
        &mut self,
        id: i32,
        count: i32,
        tables: &GameTables,
        now: i32,
        player_exp_per_stamina: i32,
    ) -> Result<DungeonSweepOutcome, DungeonError> {
        if count <= 0 {
            return Err(DungeonError::InvalidSweepCount);
        }
        if !self.completed_dungeons.contains(&id) {
            return Err(DungeonError::NotCompleted(id));
        }
        let _ = self.refresh_heat(tables, now);
        let config = tables
            .dungeon_ports
            .get(id)
            .ok_or(DungeonError::Unknown(id))?
            .clone();
        if config.sweep == 0 {
            return Err(DungeonError::SweepDisabled(id));
        }
        self.validate_dungeon_cost(&config, count)?;
        let previous_archives = self.unlocked_archive_ids(tables, now);
        let (remains, heat_update) = self.consume_dungeon_cost(&config, count, tables, now);
        let infos = (0..count)
            .map(|_| {
                let rewards = self.grant_dungeon_rewards(&config, 1, tables, now);
                let player_exp = dungeon_player_exp(&config, 1, tables, player_exp_per_stamina);
                DcNetDataDungeonSweepInfo {
                    rewards: Some(rewards),
                    user_up_data: (player_exp > 0).then(|| self.add_user_exp(player_exp, tables)),
                    ..Default::default()
                }
            })
            .collect();
        Ok(DungeonSweepOutcome {
            infos,
            remains,
            heat_update,
            archive_updates: self.newly_unlocked_archives_since(&previous_archives, tables, now),
        })
    }

    fn require_dungeon_config<'a>(
        &self,
        id: i32,
        dungeon_type: i32,
        tables: &'a GameTables,
        now: i32,
    ) -> Result<&'a configs::tables::DungeonPort, DungeonError> {
        let config = tables
            .dungeon_ports
            .get(id)
            .ok_or(DungeonError::Unknown(id))?;
        let expected = tables
            .dungeon_port_groups
            .get(config.port_group_id)
            .and_then(|group| tables.dungeons.get(group.dungeon_id))
            .map_or(0, |dungeon| dungeon.dungeon_type);
        if expected != dungeon_type {
            return Err(DungeonError::WrongType {
                id,
                expected,
                actual: dungeon_type,
            });
        }
        let unlocked = self.level >= config.level_need
            && config.unlock_conditions.iter().all(|limit| {
                self.condition_progress(limit.key, &limit.value, tables, now)
                    >= condition_total(limit.key, &limit.value)
            });
        if !unlocked {
            return Err(DungeonError::Locked(id));
        }
        Ok(config)
    }

    fn validate_multiplier(&self, multiplier: i32, maximum: i32) -> Result<(), DungeonError> {
        if (1..=maximum.max(1)).contains(&multiplier) {
            Ok(())
        } else {
            Err(DungeonError::InvalidMultiplier {
                maximum: maximum.max(1),
                actual: multiplier,
            })
        }
    }

    fn validate_dungeon_cost(
        &self,
        config: &configs::tables::DungeonPort,
        multiplier: i32,
    ) -> Result<(), DungeonError> {
        for cost in &config.cost {
            let needed = cost.value.saturating_mul(multiplier);
            let available = self
                .items
                .iter()
                .find(|item| item.item_id == cost.key)
                .map_or(0, |item| item.amount);
            if available < needed {
                return Err(DungeonError::InsufficientCost {
                    item_id: cost.key,
                    needed,
                    available,
                });
            }
        }
        Ok(())
    }

    fn consume_dungeon_cost(
        &mut self,
        config: &configs::tables::DungeonPort,
        multiplier: i32,
        tables: &GameTables,
        now: i32,
    ) -> (Vec<DcNetDataItem>, Option<HeatInfoOutcome>) {
        let mut remains = Vec::new();
        let mut heat_update = None;
        for cost in &config.cost {
            let amount = cost.value.saturating_mul(multiplier);
            if let Some(item) = self.items.iter_mut().find(|item| item.item_id == cost.key) {
                let maximum = tables
                    .player_levels
                    .get(self.level)
                    .map_or(0, |level| level.max_stamina);
                let starts_heat_regeneration = cost.key
                    == tables.cultivation_constants.heat_item_id
                    && maximum > 0
                    && item.amount >= maximum
                    && item.amount.saturating_sub(amount) < maximum;
                if starts_heat_regeneration {
                    self.heat_updated_at = now;
                }
                item.amount -= amount;
                remains.push(*item);
                if starts_heat_regeneration {
                    heat_update = Some(HeatInfoOutcome {
                        item: *item,
                        cd_time: tables.cultivation_constants.heat_regeneration_interval,
                    });
                }
                let spent = self.item_spent.entry(cost.key).or_default();
                *spent = spent.saturating_add(amount);
            }
        }
        (remains, heat_update)
    }

    fn grant_dungeon_rewards(
        &mut self,
        config: &configs::tables::DungeonPort,
        multiplier: i32,
        tables: &GameTables,
        now: i32,
    ) -> DcNetDataTakeRewardRes {
        let mut rewards = DcNetDataTakeRewardRes::default();
        for reward in &config.guaranteed_rewards {
            self.add_reward(
                reward.key,
                reward.value.saturating_mul(multiplier),
                tables,
                now,
                &mut rewards,
            );
        }
        for _ in 0..multiplier.max(0) {
            if let Some(reward) = select_possible_reward(&config.possible_rewards, tables) {
                self.add_reward(reward.key, reward.value, tables, now, &mut rewards);
            }
        }
        rewards
    }
}

fn select_possible_reward<'a>(
    rewards: &'a [configs::tables::TablePair<i32>],
    tables: &GameTables,
) -> Option<&'a configs::tables::TablePair<i32>> {
    if rewards.is_empty() {
        return None;
    }
    if !rewards.iter().all(|reward| {
        tables
            .items
            .get(reward.key)
            .is_some_and(|item| item.item_type == 102 && item.sub_type == 11)
    }) {
        return rewards.get(fastrand::usize(..rewards.len()));
    }

    let total_weight: u32 = RELIC_QUALITY_WEIGHTS
        .iter()
        .filter(|(quality, _)| {
            rewards.iter().any(|reward| {
                tables
                    .items
                    .get(reward.key)
                    .is_some_and(|item| item.quality == *quality)
            })
        })
        .map(|(_, weight)| weight)
        .sum();
    let quality = select_relic_quality(rewards, tables, fastrand::u32(..total_weight))?;
    let candidates = rewards.iter().filter(|reward| {
        tables
            .items
            .get(reward.key)
            .is_some_and(|item| item.quality == quality)
    });
    let count = candidates.clone().count();
    candidates.into_iter().nth(fastrand::usize(..count))
}

fn select_relic_quality(
    rewards: &[configs::tables::TablePair<i32>],
    tables: &GameTables,
    mut roll: u32,
) -> Option<i32> {
    for (quality, weight) in RELIC_QUALITY_WEIGHTS {
        let available = rewards.iter().any(|reward| {
            tables
                .items
                .get(reward.key)
                .is_some_and(|item| item.quality == quality)
        });
        if !available {
            continue;
        }
        if roll < weight {
            return Some(quality);
        }
        roll -= weight;
    }
    None
}

fn dungeon_player_exp(
    config: &configs::tables::DungeonPort,
    multiplier: i32,
    tables: &GameTables,
    player_exp_per_stamina: i32,
) -> i32 {
    config
        .cost
        .iter()
        .filter(|cost| cost.key == tables.cultivation_constants.heat_item_id)
        .map(|cost| cost.value.saturating_mul(multiplier))
        .sum::<i32>()
        .saturating_mul(player_exp_per_stamina.max(0))
}
#[cfg(test)]
mod tests;
