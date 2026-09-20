use super::*;
use protocol::pbcommon::{DcNetDataMoreChallengeInfo, DcNetDataMoreTeamGroup};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoreTeamChallengeState {
    pub info: DcNetDataMoreChallengeInfo,
    pub cycle: i32,
}

impl Player {
    pub fn more_team_challenge_groups(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> (Vec<DcNetDataMoreTeamGroup>, bool) {
        let changed = self.sync_more_team_cycles(tables, now, zone_offset);
        let groups = tables
            .more_team_challenge_groups
            .rows
            .iter()
            .filter(|group| self.more_team_group_unlocked(group, tables))
            .filter_map(|group| {
                group_cycle(group, now, zone_offset).map(|(_, countdown)| DcNetDataMoreTeamGroup {
                    gid: group.id,
                    countdown,
                })
            })
            .collect();
        (groups, changed)
    }

    pub fn more_team_challenges(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> (Vec<DcNetDataMoreChallengeInfo>, bool) {
        let changed = self.sync_more_team_cycles(tables, now, zone_offset);
        (
            self.more_team_challenges
                .iter()
                .map(|state| state.info.clone())
                .collect(),
            changed,
        )
    }

    pub fn start_more_team_challenge(
        &mut self,
        id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<bool, MoreTeamChallengeError> {
        let changed = self.sync_more_team_cycles(tables, now, zone_offset);
        self.ensure_more_team_challenge_available(id, tables, now, zone_offset)?;
        Ok(changed)
    }

    pub fn settle_more_team_challenge(
        &mut self,
        id: i32,
        cost_time: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<DcNetDataMoreChallengeInfo, MoreTeamChallengeError> {
        if cost_time <= 0 {
            return Err(MoreTeamChallengeError::InvalidTime(id));
        }
        self.sync_more_team_cycles(tables, now, zone_offset);
        let (challenge, cycle) = self
            .ensure_more_team_challenge_available(id, tables, now, zone_offset)
            .map(|(challenge, cycle)| (challenge.clone(), cycle))?;
        let stars = [
            cost_time <= challenge.star_limit_1.value,
            cost_time <= challenge.star_limit_2.value,
            cost_time <= challenge.star_limit_3.value,
        ];
        let index = self
            .more_team_challenges
            .iter()
            .position(|state| state.info.id == id)
            .unwrap_or_else(|| {
                self.more_team_challenges.push(MoreTeamChallengeState {
                    info: DcNetDataMoreChallengeInfo {
                        id,
                        cost_time,
                        ..Default::default()
                    },
                    cycle,
                });
                self.more_team_challenges.len() - 1
            });
        let info = &mut self.more_team_challenges[index].info;
        info.cost_time = if info.cost_time == 0 {
            cost_time
        } else {
            info.cost_time.min(cost_time)
        };
        info.star1 |= stars[0];
        info.star2 |= stars[1];
        info.star3 |= stars[2];
        Ok(info.clone())
    }

    pub fn claim_more_team_challenge_rewards(
        &mut self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> (DcNetDataTakeRewardRes, Vec<i32>, bool) {
        let mut changed = self.sync_more_team_cycles(tables, now, zone_offset);
        let claims = tables
            .more_team_challenge_stars
            .iter()
            .filter_map(|reward| {
                let state = self
                    .more_team_challenges
                    .iter()
                    .find(|state| state.info.id == reward.dungeon_id)?;
                (more_team_star_count(&state.info) >= reward.star_limit
                    && !state.info.received_rewards.contains(&reward.id))
                .then(|| (reward.id, reward.dungeon_id, reward.reward.clone()))
            })
            .collect::<Vec<_>>();
        let mut result = DcNetDataTakeRewardRes::default();
        let mut ids = Vec::with_capacity(claims.len());
        for (reward_id, dungeon_id, rewards) in claims {
            for reward in rewards {
                self.add_reward(reward.key, reward.value, tables, now, &mut result);
            }
            self.more_team_challenges
                .iter_mut()
                .find(|state| state.info.id == dungeon_id)
                .unwrap()
                .info
                .received_rewards
                .push(reward_id);
            ids.push(reward_id);
            changed = true;
        }
        for state in &mut self.more_team_challenges {
            state.info.received_rewards.sort_unstable();
        }
        ids.sort_unstable();
        (result, ids, changed)
    }

    fn ensure_more_team_challenge_available<'a>(
        &self,
        id: i32,
        tables: &'a GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(&'a configs::tables::MoreTeamChallenge, i32), MoreTeamChallengeError> {
        let challenge = tables
            .more_team_challenges
            .get(id)
            .ok_or(MoreTeamChallengeError::Unknown(id))?;
        let group = tables
            .more_team_challenge_groups
            .get(challenge.group_id)
            .ok_or(MoreTeamChallengeError::Closed(challenge.group_id))?;
        if !self.more_team_group_unlocked(group, tables) {
            return Err(MoreTeamChallengeError::Closed(group.id));
        }
        let (cycle, _) =
            group_cycle(group, now, zone_offset).ok_or(MoreTeamChallengeError::Closed(group.id))?;
        if challenge.pre_dungeon_id != 0
            && !self
                .more_team_challenges
                .iter()
                .any(|state| state.info.id == challenge.pre_dungeon_id)
        {
            return Err(MoreTeamChallengeError::Locked(id));
        }
        let port = tables
            .more_team_challenge_ports
            .get(id)
            .filter(|port| port.dungeon_id == id)
            .ok_or(MoreTeamChallengeError::MissingPort(id))?;
        if self.level < port.level_need {
            return Err(MoreTeamChallengeError::LevelLocked {
                id,
                required: port.level_need,
            });
        }
        Ok((challenge, cycle))
    }

    fn more_team_group_unlocked(
        &self,
        group: &configs::tables::MoreTeamChallengeGroup,
        tables: &GameTables,
    ) -> bool {
        group.limit_type.iter().all(|limit| {
            let values = limit
                .value
                .split('|')
                .filter_map(|value| value.parse::<i32>().ok())
                .collect::<Vec<_>>();
            match (limit.key, values.as_slice()) {
                (305, [level, world]) => self.level >= *level && self.world_level(tables) >= *world,
                (306, [group_id, stars]) => self.more_team_group_stars(*group_id, tables) >= *stars,
                _ => false,
            }
        })
    }

    fn more_team_group_stars(&self, group_id: i32, tables: &GameTables) -> i32 {
        self.more_team_challenges
            .iter()
            .filter(|state| {
                tables
                    .more_team_challenges
                    .get(state.info.id)
                    .is_some_and(|challenge| challenge.group_id == group_id)
            })
            .map(|state| more_team_star_count(&state.info))
            .sum()
    }

    fn sync_more_team_cycles(&mut self, tables: &GameTables, now: i32, zone_offset: i32) -> bool {
        let before = self.more_team_challenges.len();
        self.more_team_challenges.retain(|state| {
            tables
                .more_team_challenges
                .get(state.info.id)
                .and_then(|challenge| tables.more_team_challenge_groups.get(challenge.group_id))
                .and_then(|group| group_cycle(group, now, zone_offset))
                .is_some_and(|(cycle, _)| cycle == state.cycle)
        });
        before != self.more_team_challenges.len()
    }
}

fn group_cycle(
    group: &configs::tables::MoreTeamChallengeGroup,
    now: i32,
    zone_offset: i32,
) -> Option<(i32, i32)> {
    if group.refresh_time.is_empty() {
        return Some((-1, -1));
    }
    let (start, interval) = group.refresh_time.split_once(',')?;
    let start = common::time::table_time_utc(start, zone_offset)?;
    let close = common::time::table_time_utc(&group.close_time, zone_offset)?;
    let interval = interval.parse::<i64>().ok()?;
    let elapsed = i64::from(now) - start;
    if interval <= 0 || elapsed < 0 || i64::from(now) >= close {
        return None;
    }
    let cycle = elapsed / interval;
    let countdown = interval - elapsed.rem_euclid(interval);
    Some((
        cycle.min(i64::from(i32::MAX)) as i32,
        countdown.min(i64::from(i32::MAX)) as i32,
    ))
}

fn more_team_star_count(info: &DcNetDataMoreChallengeInfo) -> i32 {
    i32::from(info.star1) + i32::from(info.star2) + i32::from(info.star3)
}
#[cfg(test)]
mod tests;
