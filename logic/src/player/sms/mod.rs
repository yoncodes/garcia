use super::*;

const TASK_COMPLETED_CONDITION: i32 = 81;
const TASK_PICKED_CONDITION: i32 = 84;

impl Player {
    pub fn sync_sms(&mut self, tables: &GameTables) -> Vec<DcNetDataSms> {
        let unlocked: Vec<_> = tables
            .character_message_groups
            .rows
            .iter()
            .filter(|group| {
                group.limit_type.iter().all(|limit| match limit.key {
                    TASK_COMPLETED_CONDITION => limit.value.split('|').all(|id| {
                        id.parse::<i32>().is_ok_and(|id| {
                            self.tasks
                                .iter()
                                .any(|task| task.id == id && task.status == TaskStatus::Done as i32)
                        })
                    }),
                    TASK_PICKED_CONDITION => limit.value.parse::<i32>().is_ok_and(|id| {
                        self.tasks
                            .iter()
                            .any(|task| task.id == id && task.status == TaskStatus::Picked as i32)
                    }),
                    _ => false,
                })
            })
            .map(|group| group.id)
            .filter(|id| !self.sms.iter().any(|message| message.id == *id))
            .collect();
        let updates = unlocked
            .into_iter()
            .map(|id| DcNetDataSms {
                id,
                read: false,
                selected: Vec::new(),
            })
            .collect::<Vec<_>>();
        self.sms.extend(updates.iter().cloned());
        self.sms.sort_unstable_by_key(|message| message.id);
        updates
    }

    pub fn read_sms(
        &mut self,
        id: i32,
        selected: Vec<i32>,
        tables: &GameTables,
    ) -> Result<(DcNetDataSms, bool), SmsError> {
        if let Some(&selection) = selected
            .iter()
            .find(|&&selection| !(1..=4).contains(&selection))
        {
            return Err(SmsError::InvalidSelection(selection));
        }
        self.sync_sms(tables);
        let message = self
            .sms
            .iter_mut()
            .find(|message| message.id == id)
            .ok_or(SmsError::Locked(id))?;
        let changed = !message.read || message.selected != selected;
        message.read = true;
        message.selected = selected;
        Ok((message.clone(), changed))
    }
}
#[cfg(test)]
mod tests;
