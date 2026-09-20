use std::collections::{BTreeSet, HashSet};

use super::super::*;

#[derive(Debug, Default)]
pub struct ProgressionOutcome {
    pub level_update: Option<DcNetDataAttrUpData>,
    pub mission_updates: Vec<DcNetDataMissonInfo>,
    pub tasks: Vec<DcNetDataTaskStatus>,
    pub completions: Vec<TaskCompleteOutcome>,
    pub ports: Vec<DcNetDataPort>,
    pub features: Vec<DcNetDataFeat>,
}

impl Player {
    pub fn increase_player_level(
        &mut self,
        target: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<ProgressionOutcome, ProgressionError> {
        let update = self.raise_player_level(target, tables)?;
        Ok(ProgressionOutcome {
            level_update: update,
            mission_updates: self.advance_level_missions(now),
            features: self.advance_features(tables),
            ..Default::default()
        })
    }

    pub fn increase_world_level(
        &mut self,
        target: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<ProgressionOutcome, ProgressionError> {
        let current = self.world_level(tables);
        if target < current {
            return Err(ProgressionError::WorldLevelDecrease {
                current,
                requested: target,
            });
        }
        let target_row = tables
            .world_levels
            .iter()
            .find(|row| row.level == target)
            .ok_or(ProgressionError::UnknownWorldLevel(target))?;
        self.advance_task_chain(target_row.task_id, TaskStatus::Done as i32, tables, now)
    }

    pub fn complete_story_stage(
        &mut self,
        gameplay_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<ProgressionOutcome, ProgressionError> {
        tables
            .main_story_ports
            .get(gameplay_id)
            .ok_or(ProgressionError::UnknownStoryStage(gameplay_id))?;
        let task_id = tables
            .tasks
            .rows
            .iter()
            .find(|task| {
                task.task_type == 1
                    && task.done_key.iter().any(|condition| {
                        condition.key == 39
                            && condition.value.split('|').next() == Some(&gameplay_id.to_string())
                    })
            })
            .map(|task| task.id)
            .ok_or(ProgressionError::UnlinkedStoryStage(gameplay_id))?;
        self.advance_task_chain(task_id, TaskStatus::Done as i32, tables, now)
    }

    pub(crate) fn advance_task_chain(
        &mut self,
        target_task: i32,
        target_status: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<ProgressionOutcome, ProgressionError> {
        let mut ordered = Vec::new();
        let mut visiting = HashSet::new();
        collect_task_chain(target_task, tables, &mut visiting, &mut ordered)?;

        let required_level = ordered
            .iter()
            .filter_map(|id| tables.tasks.get(*id))
            .flat_map(|task| task.show_key.iter().chain(&task.done_key))
            .filter(|condition| condition.key == 5)
            .filter_map(|condition| condition.value.parse::<i32>().ok())
            .max()
            .unwrap_or(self.level);
        let level_update = self.raise_player_level(required_level.max(self.level), tables)?;

        let mut outcome = ProgressionOutcome {
            level_update,
            ..Default::default()
        };
        for task_id in ordered {
            let task = tables
                .tasks
                .get(task_id)
                .ok_or(ProgressionError::UnknownTask(task_id))?;
            let status = if task_id == target_task {
                target_status
            } else {
                TaskStatus::Done as i32
            };
            let current_status = self
                .tasks
                .iter()
                .find(|stored| stored.id == task_id)
                .map_or(TaskStatus::Disabled as i32, |stored| stored.status);
            if current_status >= status {
                continue;
            }
            let activated = self
                .activate_task(task_id, tables, now)
                .map_err(|error| ProgressionError::Task(error.to_string()))?;
            if status == TaskStatus::Picked as i32 {
                outcome.tasks.push(activated);
                continue;
            }
            for condition in &task.done_key {
                outcome
                    .ports
                    .extend(self.apply_completion_condition(task_id, condition, tables)?);
            }
            let completion = self
                .complete_task(task_id, 0, tables, now)
                .map_err(|error| ProgressionError::Task(error.to_string()))?;
            outcome.tasks.push(completion.current);
            outcome.tasks.extend(completion.changed.iter().copied());
            outcome
                .features
                .extend(completion.feat_updates.iter().copied());
            outcome.completions.push(completion);
        }
        outcome.features.extend(self.advance_features(tables));
        let unlocked_levels = self.add_user_exp(0, tables);
        if unlocked_levels.is_lv_up {
            outcome.level_update = Some(unlocked_levels);
        }
        outcome.mission_updates = self.advance_level_missions(now);
        outcome.tasks.sort_unstable_by_key(|task| task.id);
        outcome.tasks.dedup_by_key(|task| task.id);
        outcome.ports.sort_unstable_by_key(|port| port.id);
        outcome.ports.dedup_by_key(|port| port.id);
        outcome.features.sort_unstable_by_key(|feature| feature.id);
        outcome.features.dedup_by_key(|feature| feature.id);
        Ok(outcome)
    }

    pub fn player_level_snapshot(
        &self,
        tables: &GameTables,
        apply_on_client: bool,
    ) -> DcNetDataAttrUpData {
        let item = self
            .items
            .iter()
            .find(|item| item.item_id == PLAYER_EXP_ITEM_ID)
            .copied()
            .unwrap_or_else(|| DcNetDataItem {
                item_id: PLAYER_EXP_ITEM_ID,
                quality: tables
                    .items
                    .get(PLAYER_EXP_ITEM_ID)
                    .map_or(0, |item| item.quality),
                ..Default::default()
            });
        DcNetDataAttrUpData {
            attr_lv: self.level,
            attr_exp: item.amount,
            items: vec![item],
            is_lv_up: apply_on_client,
            ..Default::default()
        }
    }

    fn raise_player_level(
        &mut self,
        target: i32,
        tables: &GameTables,
    ) -> Result<Option<DcNetDataAttrUpData>, ProgressionError> {
        if tables.player_levels.get(target).is_none() {
            return Err(ProgressionError::UnknownPlayerLevel(target));
        }
        if target < self.level {
            return Err(ProgressionError::PlayerLevelDecrease {
                current: self.level,
                requested: target,
            });
        }
        if target == self.level {
            return Ok(None);
        }
        let current_exp = self
            .items
            .iter()
            .find(|item| item.item_id == PLAYER_EXP_ITEM_ID)
            .map_or(0, |item| item.amount);
        let needed = (self.level..target)
            .map(|level| tables.player_levels.get(level).map_or(0, |row| row.exp))
            .sum::<i32>()
            .saturating_sub(current_exp);
        Ok(Some(self.add_user_exp(needed, tables)))
    }

    fn apply_completion_condition(
        &mut self,
        task_id: i32,
        condition: &configs::tables::TablePair<String>,
        tables: &GameTables,
    ) -> Result<Vec<DcNetDataPort>, ProgressionError> {
        match condition.key {
            39 | 40 => {
                let (gameplay, count) = condition
                    .value
                    .split_once('|')
                    .ok_or_else(|| invalid_condition(task_id, condition))?;
                let gameplay = gameplay
                    .parse::<i32>()
                    .map_err(|_| invalid_condition(task_id, condition))?;
                let count = count
                    .parse::<i32>()
                    .map_err(|_| invalid_condition(task_id, condition))?;
                tables
                    .main_story_ports
                    .get(gameplay)
                    .ok_or(ProgressionError::UnknownStoryStage(gameplay))?;
                let mut updates = Vec::new();
                loop {
                    let progress =
                        self.ports
                            .iter()
                            .find(|port| port.id == gameplay)
                            .map_or(0, |port| {
                                if condition.key == 39 {
                                    port.pass_cnt
                                } else {
                                    port.all_cnt
                                }
                            });
                    if progress >= count {
                        break;
                    }
                    self.start_port(gameplay, tables)
                        .map_err(|error| ProgressionError::Port(error.to_string()))?;
                    let (port, _) = self
                        .settle_port(
                            &DcNetDataParamsSettlement {
                                id: gameplay,
                                is_completed: i32::from(condition.key == 39),
                                stars: if condition.key == 39 {
                                    vec![true; 3]
                                } else {
                                    Vec::new()
                                },
                                ..Default::default()
                            },
                            tables,
                        )
                        .map_err(|error| ProgressionError::Port(error.to_string()))?;
                    updates.push(port);
                }
                Ok(updates)
            }
            83 => {
                let (object, status) = condition
                    .value
                    .rsplit_once('|')
                    .ok_or_else(|| invalid_condition(task_id, condition))?;
                let status = status
                    .parse::<i32>()
                    .map_err(|_| invalid_condition(task_id, condition))?;
                self.record_interaction(object, status, tables)
                    .map_err(|error| ProgressionError::Interaction(error.to_string()))?;
                Ok(Vec::new())
            }
            // These task conditions are completed by the corresponding client action and
            // have no separate durable value. The task status is the authoritative state.
            5 | 80 | 87 => Ok(Vec::new()),
            condition => Err(ProgressionError::UnsupportedCondition { task_id, condition }),
        }
    }
}

fn collect_task_chain(
    task_id: i32,
    tables: &GameTables,
    visiting: &mut HashSet<i32>,
    ordered: &mut Vec<i32>,
) -> Result<(), ProgressionError> {
    if ordered.contains(&task_id) || !visiting.insert(task_id) {
        return Ok(());
    }
    let task = tables
        .tasks
        .get(task_id)
        .ok_or(ProgressionError::UnknownTask(task_id))?;
    let mut predecessors = BTreeSet::new();
    for condition in &task.show_key {
        if matches!(condition.key, 81 | 84) {
            predecessors.extend(
                condition
                    .value
                    .split('|')
                    .filter_map(|value| value.parse::<i32>().ok()),
            );
        }
    }
    for candidate in &tables.tasks.rows {
        if candidate
            .next
            .iter()
            .flat_map(|next| next.split('|'))
            .any(|next| next.parse::<i32>().ok() == Some(task_id))
        {
            predecessors.insert(candidate.id);
        }
    }
    for predecessor in predecessors {
        collect_task_chain(predecessor, tables, visiting, ordered)?;
    }
    visiting.remove(&task_id);
    ordered.push(task_id);
    Ok(())
}

fn invalid_condition(
    task_id: i32,
    condition: &configs::tables::TablePair<String>,
) -> ProgressionError {
    ProgressionError::InvalidCondition {
        task_id,
        condition: condition.key,
        value: condition.value.clone(),
    }
}
