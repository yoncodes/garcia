use super::*;

impl Player {
    pub fn claim_daily_activity(
        &mut self,
        id: i32,
        tables: &GameTables,
    ) -> Result<(i32, i32, i32), DailyTaskError> {
        let reward = tables
            .daily_tasks
            .get(id)
            .ok_or(DailyTaskError::Unknown(id))?
            .reward;
        let task = self
            .daily_tasks
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or(DailyTaskError::Unknown(id))?;
        if task.taken {
            return Err(DailyTaskError::AlreadyTaken(id));
        }
        if task.progress < task.total {
            return Err(DailyTaskError::Incomplete(id));
        }
        task.taken = true;
        self.daily_activity = self.daily_activity.saturating_add(reward);
        Ok((id, self.daily_activity, self.daily_reward_progress))
    }

    pub fn claim_daily_rewards(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<DailyRewardOutcome, DailyTaskError> {
        let mut tiers: Vec<_> = tables
            .daily_activity_milestones
            .rows
            .iter()
            .filter(|tier| {
                tier.id > self.daily_reward_progress && tier.active <= self.daily_activity
            })
            .cloned()
            .collect();
        tiers.sort_by_key(|tier| tier.id);
        let Some(last) = tiers.last() else {
            return Err(DailyTaskError::NoReward);
        };
        let previous_archives = self.unlocked_archive_ids(tables, now);
        self.daily_reward_progress = last.id;

        let mut item_rewards = BTreeMap::new();
        let mut player_exp: i32 = 0;
        for tier in tiers {
            for reward in tier.active_reward {
                let amount = item_rewards.entry(reward.key).or_insert(0i32);
                *amount = amount.saturating_add(reward.value);
            }
            player_exp = player_exp.saturating_add(tier.player_exp.value);
        }
        let mut rewards = DcNetDataTakeRewardRes::default();
        for (id, amount) in item_rewards {
            self.add_reward(id, amount, tables, now, &mut rewards);
        }
        let level_up_data = self.add_user_exp(player_exp, tables);
        Ok(DailyRewardOutcome {
            rewards,
            level_up_data,
            archive_updates: self.newly_unlocked_archives_since(&previous_archives, tables, now),
        })
    }
}
#[cfg(test)]
mod tests;
