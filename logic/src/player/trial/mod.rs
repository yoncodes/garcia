use protocol::pbcommon::DcNetDataTrial;

use super::*;

impl Player {
    pub fn trials(&self, tables: &GameTables, now: i32) -> Vec<DcNetDataTrial> {
        tables
            .trials
            .rows
            .iter()
            .filter(|trial| {
                trial.unlock_conditions.iter().all(|limit| {
                    self.condition_progress(limit.key, &limit.value, tables, now)
                        >= condition_total(limit.key, &limit.value)
                })
            })
            .map(|trial| DcNetDataTrial { trial_id: trial.id })
            .collect()
    }

    pub fn trial_info(
        &self,
        trial_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTrial, TrialError> {
        self.unlocked_trial(trial_id, tables, now)?;
        Ok(DcNetDataTrial { trial_id })
    }

    pub fn save_trial(
        &self,
        trial_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<Vec<i32>, TrialError> {
        self.unlocked_trial(trial_id, tables, now)?;
        let roles = self
            .formations
            .iter()
            .find(|formation| formation.formation_id == self.cur_form)
            .map(|formation| {
                formation
                    .poss
                    .iter()
                    .map(|position| position.game_role_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if roles.is_empty() {
            return Err(TrialError::EmptyFormation);
        }
        Ok(roles)
    }

    pub fn finish_trial(
        &mut self,
        trial_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<TrialDoneOutcome, TrialError> {
        let config = self.unlocked_trial(trial_id, tables, now)?.clone();
        if self
            .tasks
            .iter()
            .any(|task| task.id == config.finish_task_id && task.status == TaskStatus::Done as i32)
        {
            return Err(TrialError::AlreadyComplete(trial_id));
        }

        let mut reward = DcNetDataTakeRewardRes::default();
        for entry in config.reward {
            self.add_reward(entry.key, entry.value, tables, now, &mut reward);
        }

        let mut task = self
            .tasks
            .iter()
            .find(|task| task.id == config.finish_task_id)
            .copied()
            .unwrap_or(DcNetDataTaskStatus {
                id: config.finish_task_id,
                picked_at: i64::from(now),
                ..Default::default()
            });
        task.status = TaskStatus::Done as i32;
        let task = self.store_task_status(task);

        let local = if config.complete_point.is_empty() {
            self.locals
                .iter()
                .find(|local| local.region == trial_id)
                .cloned()
        } else {
            let local = DcNetDataLocal {
                region: trial_id,
                local: config.complete_point,
                local2: String::new(),
            };
            self.set_local(local.clone());
            Some(local)
        };

        Ok(TrialDoneOutcome {
            trial_id,
            reward,
            local,
            task,
        })
    }

    fn unlocked_trial<'a>(
        &self,
        trial_id: i32,
        tables: &'a GameTables,
        now: i32,
    ) -> Result<&'a configs::tables::Trial, TrialError> {
        let trial = tables
            .trials
            .get(trial_id)
            .ok_or(TrialError::Unknown(trial_id))?;
        if trial.unlock_conditions.iter().all(|limit| {
            self.condition_progress(limit.key, &limit.value, tables, now)
                >= condition_total(limit.key, &limit.value)
        }) {
            Ok(trial)
        } else {
            Err(TrialError::Locked(trial_id))
        }
    }
}
#[cfg(test)]
mod tests;
