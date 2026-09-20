use super::*;
use protocol::pbcommon::DcNetDataVersionChallengeStage;

impl Player {
    pub fn version_challenge_stages(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataVersionChallengeStage> {
        let Some((activity_id, _)) = self.active_version_activity(tables, now, zone_offset) else {
            return Vec::new();
        };
        tables
            .version_challenges
            .rows
            .iter()
            .filter(|challenge| challenge.activity_id == activity_id)
            .map(|challenge| DcNetDataVersionChallengeStage {
                id: challenge.id,
                countdown: challenge_countdown(challenge, now, zone_offset),
            })
            .collect()
    }

    pub fn version_challenges(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataVersionChallengeInfo> {
        let Some((activity_id, _)) = self.active_version_activity(tables, now, zone_offset) else {
            return Vec::new();
        };
        self.version_challenges
            .iter()
            .filter(|state| {
                tables
                    .version_challenges
                    .get(state.id)
                    .is_some_and(|challenge| challenge.activity_id == activity_id)
            })
            .cloned()
            .collect()
    }

    pub fn start_version_challenge(
        &self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(), VersionChallengeError> {
        self.ensure_version_challenge_available(id, tables, now, zone_offset)?;
        Ok(())
    }

    pub fn settle_version_challenge(
        &mut self,
        id: i32,
        pass_cond: &HashMap<i32, i32>,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataVersionChallengeInfo, VersionChallengeError> {
        let challenge = self
            .ensure_version_challenge_available(id, tables, now, zone_offset)?
            .clone();
        if !(1..=3).all(|key| pass_cond.get(&key).is_some_and(|value| *value >= 0)) {
            return Err(VersionChallengeError::InvalidResult(id));
        }
        let earned = [
            is_star_requirement_met(&challenge.star_limit_1, pass_cond),
            is_star_requirement_met(&challenge.star_limit_2, pass_cond),
            is_star_requirement_met(&challenge.star_limit_3, pass_cond),
        ];
        let index = self
            .version_challenges
            .iter()
            .position(|state| state.id == id)
            .unwrap_or_else(|| {
                self.version_challenges.push(DcNetDataVersionChallengeInfo {
                    id,
                    ..Default::default()
                });
                self.version_challenges.len() - 1
            });
        let state = &mut self.version_challenges[index];
        state.star1 |= earned[0];
        state.star2 |= earned[1];
        state.star3 |= earned[2];
        Ok(state.clone())
    }

    pub fn claim_version_challenge_rewards(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataTakeRewardRes, Vec<i32>), VersionChallengeError> {
        let (activity_id, _) = self
            .active_version_activity(tables, now, zone_offset)
            .ok_or(VersionChallengeError::Inactive)?;
        let claims = tables
            .version_challenge_rewards
            .iter()
            .filter_map(|reward| {
                let challenge = tables.version_challenges.get(reward.dungeon_id)?;
                let state = self
                    .version_challenges
                    .iter()
                    .find(|state| state.id == reward.dungeon_id)?;
                (challenge.activity_id == activity_id
                    && star_count(state) >= reward.star_limit
                    && !state.received_rewards.contains(&reward.id))
                .then(|| (reward.id, reward.dungeon_id, reward.reward.clone()))
            })
            .collect::<Vec<_>>();
        let mut result = DcNetDataTakeRewardRes::default();
        let mut ids = Vec::with_capacity(claims.len());
        for (reward_id, dungeon_id, rewards) in claims {
            for reward in rewards {
                self.add_reward(reward.key, reward.value, tables, now, &mut result);
            }
            self.version_challenges
                .iter_mut()
                .find(|state| state.id == dungeon_id)
                .unwrap()
                .received_rewards
                .push(reward_id);
            ids.push(reward_id);
        }
        for state in &mut self.version_challenges {
            state.received_rewards.sort_unstable();
        }
        ids.sort_unstable();
        Ok((result, ids))
    }

    fn ensure_version_challenge_available<'a>(
        &self,
        id: i32,
        tables: &'a GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<&'a configs::tables::VersionChallenge, VersionChallengeError> {
        let challenge = tables
            .version_challenges
            .get(id)
            .ok_or(VersionChallengeError::Unknown(id))?;
        let (activity_id, _) = self
            .active_version_activity(tables, now, zone_offset)
            .ok_or(VersionChallengeError::Inactive)?;
        if challenge.activity_id != activity_id
            || challenge_countdown(challenge, now, zone_offset) != 0
        {
            return Err(VersionChallengeError::Locked(id));
        }
        let close = common::time::table_time_utc(&challenge.close_time, zone_offset)
            .ok_or(VersionChallengeError::Locked(id))?;
        if i64::from(now) >= close {
            return Err(VersionChallengeError::Locked(id));
        }
        if challenge.pre_dungeon_id != 0
            && !self
                .version_challenges
                .iter()
                .any(|state| state.id == challenge.pre_dungeon_id)
        {
            return Err(VersionChallengeError::Locked(id));
        }
        let port = tables
            .version_challenge_ports
            .rows
            .iter()
            .find(|port| port.dungeon_id == id)
            .ok_or(VersionChallengeError::MissingPort(id))?;
        if self.level < port.level_need {
            return Err(VersionChallengeError::LevelLocked {
                id,
                required: port.level_need,
            });
        }
        Ok(challenge)
    }
}

fn challenge_countdown(
    challenge: &configs::tables::VersionChallenge,
    now: i32,
    zone_offset: i32,
) -> i32 {
    common::time::table_time_utc(&challenge.open_time, zone_offset)
        .map_or(0, |open| common::time::seconds_until(open, i64::from(now)))
}

fn is_star_requirement_met(
    limit: &configs::tables::TablePair<i32>,
    values: &HashMap<i32, i32>,
) -> bool {
    values.get(&limit.key).is_some_and(|value| match limit.key {
        1 => *value <= limit.value,
        2 | 3 => *value >= limit.value,
        _ => false,
    })
}

fn star_count(info: &DcNetDataVersionChallengeInfo) -> i32 {
    i32::from(info.star1) + i32::from(info.star2) + i32::from(info.star3)
}
#[cfg(test)]
mod tests;
