use super::*;
use protocol::pbcommon::DcNetDataActivityChallenge;

impl Player {
    pub fn activity_challenges(
        &self,
        activity_id: i32,
        tables: &GameTables,
    ) -> Result<(Vec<DcNetDataActivityChallenge>, Vec<i32>), ActivityChallengeError> {
        if !tables
            .activity_challenges
            .rows
            .iter()
            .any(|row| row.activity_id == activity_id)
        {
            return Err(ActivityChallengeError::UnknownActivity(activity_id));
        }
        let list = tables
            .activity_challenges
            .rows
            .iter()
            .filter(|row| row.activity_id == activity_id)
            .map(|row| DcNetDataActivityChallenge {
                id: row.id,
                enabled: self.is_activity_challenge_enabled(row),
                challenge_info: self
                    .challenges
                    .iter()
                    .find(|challenge| challenge.id == row.challenge_id)
                    .cloned(),
            })
            .collect();
        let claimed = self
            .activity_challenge_point_claims
            .iter()
            .filter(|id| {
                tables
                    .activity_challenge_milestones
                    .get(**id)
                    .is_some_and(|row| row.activity_id == activity_id)
            })
            .copied()
            .collect();
        Ok((list, claimed))
    }

    pub fn claim_activity_challenge(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataTakeRewardRes, ActivityChallengeError> {
        let row = tables
            .activity_challenges
            .get(id)
            .ok_or(ActivityChallengeError::UnknownChallenge(id))?;
        ensure_activity_is_active(row.activity_id, tables, now, zone_offset)?;
        if !self.is_activity_challenge_enabled(row) {
            return Err(ActivityChallengeError::Disabled(id));
        }
        let challenge = self
            .challenges
            .iter_mut()
            .find(|challenge| challenge.id == row.challenge_id)
            .ok_or(ActivityChallengeError::UnknownChallenge(id))?;
        if !challenge.finished {
            return Err(ActivityChallengeError::Incomplete(id));
        }
        if challenge.claimed {
            return Err(ActivityChallengeError::AlreadyClaimed(id));
        }
        challenge.claimed = true;
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in &row.reward {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn claim_activity_challenge_point(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataTakeRewardRes, ActivityChallengeError> {
        let row = tables
            .activity_challenge_milestones
            .get(id)
            .ok_or(ActivityChallengeError::UnknownPointReward(id))?;
        ensure_activity_is_active(row.activity_id, tables, now, zone_offset)?;
        if self.activity_challenge_point_claims.contains(&id) {
            return Err(ActivityChallengeError::PointAlreadyClaimed(id));
        }
        let claimed = tables
            .activity_challenges
            .rows
            .iter()
            .filter(|challenge| challenge.activity_id == row.activity_id)
            .filter(|challenge| {
                self.challenges
                    .iter()
                    .any(|state| state.id == challenge.challenge_id && state.claimed)
            })
            .count() as i32;
        if claimed < row.challenge_point {
            return Err(ActivityChallengeError::PointNotReached(id));
        }
        let mut result = DcNetDataTakeRewardRes::default();
        self.add_reward(row.reward.key, row.reward.value, tables, now, &mut result);
        self.activity_challenge_point_claims.push(id);
        self.activity_challenge_point_claims.sort_unstable();
        Ok(result)
    }

    fn is_activity_challenge_enabled(&self, row: &configs::tables::ActivityChallenge) -> bool {
        row.limit_open.iter().all(|limit| match limit.key {
            81 => limit.value.parse::<i32>().ok().is_some_and(|id| {
                self.tasks
                    .iter()
                    .any(|task| task.id == id && task.status == TaskStatus::Done as i32)
            }),
            90 => self
                .challenges
                .iter()
                .any(|challenge| challenge.id == limit.value && challenge.finished),
            _ => false,
        })
    }
}

fn ensure_activity_is_active(
    activity_id: i32,
    tables: &GameTables,
    now: i32,
    zone_offset: i32,
) -> Result<(), ActivityChallengeError> {
    let activity = tables
        .activities
        .get(activity_id)
        .ok_or(ActivityChallengeError::UnknownActivity(activity_id))?;
    let active = common::time::table_window_active(
        &activity.open_time,
        &activity.close_time,
        now,
        zone_offset,
    );
    if active {
        Ok(())
    } else {
        Err(ActivityChallengeError::Inactive(activity_id))
    }
}
#[cfg(test)]
mod tests;
