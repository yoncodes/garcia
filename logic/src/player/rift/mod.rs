use super::*;

const RIFT_SCORE_PER_SECOND: i32 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RiftState {
    pub id: i32,
    pub current_port_id: i32,
    pub best_buff_score: i32,
    pub best_score: i32,
    pub best_stage: i32,
    pub completion_count: i32,
    pub run_buff_score: i32,
    pub run_score: i32,
    pub run_duration: i32,
    pub active: bool,
    pub roles: Vec<i32>,
    pub tasks: Vec<i32>,
}

impl RiftState {
    pub fn task_progress(&self, condition: i32) -> i32 {
        match condition {
            301 => self.best_stage,
            302 => self.best_buff_score,
            303 => self.completion_count,
            304 => self.best_score,
            _ => 0,
        }
    }
}

impl Player {
    pub fn sync_rift(&mut self, id: i32) -> bool {
        if self.rift.id == id {
            return false;
        }
        self.rift = RiftState {
            id,
            ..Default::default()
        };
        true
    }

    pub fn start_rift(
        &mut self,
        event: &configs::tables::Rift,
        buff_map: &HashMap<i32, i32>,
        roles: Vec<i32>,
        tables: &GameTables,
    ) -> Result<(i32, Vec<DcNetDataItem>), RiftError> {
        self.sync_rift(event.id);
        let first_stage = tables
            .rift_ports
            .rows
            .iter()
            .filter(|stage| stage.group == event.gameplay_port_group)
            .min_by_key(|stage| stage.step)
            .ok_or(RiftError::Inactive(event.id))?;

        let mut buff_score = 0i32;
        for (&buff_id, &choice) in buff_map {
            let buff = tables
                .rift_buffs
                .get(buff_id)
                .filter(|buff| buff.group == event.buff_group)
                .ok_or(RiftError::UnknownBuff(buff_id))?;
            if buff.unlock_limit > self.rift.best_buff_score {
                return Err(RiftError::LockedBuff {
                    buff_id,
                    needed: buff.unlock_limit,
                });
            }
            buff.buff(choice)
                .ok_or(RiftError::UnknownBuffChoice { buff_id, choice })?;
            buff_score = buff_score.saturating_add(choice);
        }
        for role_id in &roles {
            if !self.roles.iter().any(|role| {
                role.role_basic_info
                    .as_ref()
                    .is_some_and(|role| role.game_role_id == *role_id)
            }) {
                return Err(RiftError::UnknownRole(*role_id));
            }
        }

        let ticket_id = tables.cultivation_constants.ticket_item_id;
        let entry_cost = tables.cultivation_constants.rift_entry_cost;
        let remains = consume_item_costs(
            &mut self.items,
            &BTreeMap::from([(ticket_id, entry_cost)]),
            |_, needed, available| RiftError::InsufficientTickets { needed, available },
        )?;

        self.rift.current_port_id = first_stage.id;
        self.rift.run_buff_score = buff_score;
        self.rift.run_score = 0;
        self.rift.run_duration = 0;
        self.rift.active = true;
        self.rift.roles = roles;
        self.rift.best_stage = self.rift.best_stage.max(first_stage.step);
        Ok((first_stage.id, remains))
    }

    pub fn report_rift_stage(
        &mut self,
        event: &configs::tables::Rift,
        duration: i32,
        tables: &GameTables,
    ) -> Result<(i32, i32), RiftError> {
        if duration < 0 {
            return Err(RiftError::NegativeDuration);
        }
        self.ensure_rift_is_active(event.id)?;
        let current = tables
            .rift_ports
            .get(self.rift.current_port_id)
            .filter(|stage| stage.group == event.gameplay_port_group)
            .ok_or(RiftError::UnknownStage(self.rift.current_port_id))?;
        let next = tables
            .rift_ports
            .rows
            .iter()
            .filter(|stage| stage.group == current.group && stage.step > current.step)
            .min_by_key(|stage| stage.step)
            .ok_or(RiftError::FinalStage(current.id))?;
        self.rift.run_score = self
            .rift
            .run_score
            .saturating_add(rift_stage_score(current, duration));
        self.rift.run_duration = self.rift.run_duration.saturating_add(duration);
        self.rift.current_port_id = next.id;
        self.rift.best_stage = self.rift.best_stage.max(next.step);
        Ok((self.rift.run_duration, next.id))
    }

    pub fn settle_rift(
        &mut self,
        event: &configs::tables::Rift,
        duration: i32,
        tables: &GameTables,
    ) -> Result<(i32, i32, i32, i32), RiftError> {
        if duration < 0 {
            return Err(RiftError::NegativeDuration);
        }
        self.ensure_rift_is_active(event.id)?;
        let current = tables
            .rift_ports
            .get(self.rift.current_port_id)
            .filter(|stage| stage.group == event.gameplay_port_group)
            .ok_or(RiftError::UnknownStage(self.rift.current_port_id))?;
        if tables
            .rift_ports
            .rows
            .iter()
            .any(|stage| stage.group == current.group && stage.step > current.step)
        {
            return Err(RiftError::NotFinalStage(current.id));
        }
        self.rift.run_score = self
            .rift
            .run_score
            .saturating_add(rift_stage_score(current, duration));
        self.rift.run_duration = self.rift.run_duration.saturating_add(duration);
        self.rift.completion_count = self.rift.completion_count.saturating_add(1);
        self.finish_rift();
        Ok((
            self.rift.run_duration,
            self.rift.run_score,
            current.id,
            self.rift.run_buff_score,
        ))
    }

    pub fn defeat_rift(&mut self, event_id: i32) -> Result<i32, RiftError> {
        self.ensure_rift_is_active(event_id)?;
        self.finish_rift();
        Ok(self.rift.run_score)
    }

    pub fn claim_rift_task(
        &mut self,
        event: &configs::tables::Rift,
        task_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, RiftError> {
        if self.rift.id != event.id {
            return Err(RiftError::Inactive(event.id));
        }
        let task = tables
            .rift_rewards
            .iter()
            .find(|task| task.id == task_id && task.group == event.reward_group)
            .cloned()
            .ok_or(RiftError::UnknownTask(task_id))?;
        if self.rift.tasks.contains(&task_id) {
            return Err(RiftError::TaskAlreadyClaimed(task_id));
        }
        let complete = task.finish_limit.first().is_some_and(|limit| {
            limit
                .value
                .parse::<i32>()
                .is_ok_and(|total| self.rift.task_progress(limit.key) >= total)
        });
        if !complete {
            return Err(RiftError::TaskNotComplete(task_id));
        }

        let mut result = DcNetDataTakeRewardRes::default();
        for reward in task.reward {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        self.rift.tasks.push(task_id);
        self.rift.tasks.sort_unstable();
        Ok(result)
    }

    fn ensure_rift_is_active(&self, event_id: i32) -> Result<(), RiftError> {
        if self.rift.id != event_id {
            return Err(RiftError::Inactive(event_id));
        }
        if !self.rift.active {
            return Err(RiftError::NoActiveRun);
        }
        Ok(())
    }

    fn finish_rift(&mut self) {
        self.rift.active = false;
        self.rift.best_buff_score = self.rift.best_buff_score.max(self.rift.run_buff_score);
        self.rift.best_score = self.rift.best_score.max(self.rift.run_score);
    }
}

fn rift_stage_score(stage: &configs::tables::RiftPort, duration: i32) -> i32 {
    let remaining = stage.end_time.saturating_sub(duration).max(0);
    let base = remaining.saturating_mul(RIFT_SCORE_PER_SECOND);
    if remaining > stage.extra_score_time {
        base.saturating_add(base.saturating_mul(stage.extra_score).saturating_div(100))
    } else {
        base
    }
}
#[cfg(test)]
mod tests;
