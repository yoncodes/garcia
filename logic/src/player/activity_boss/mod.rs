use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityBossState {
    pub aid: i32,
    pub day: i32,
    pub damage: i64,
    pub daily_damage: i64,
    pub role_ids: Vec<i32>,
    pub rewards: Vec<i32>,
    pub daily_rewards: BTreeMap<i32, i32>,
}

impl Player {
    pub fn activity_boss_info(
        &mut self,
        aid: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(DcNetDataActivityBossInfo, bool, i32, bool), ActivityBossError> {
        let (activity, boss) = require_activity_boss_config(aid, tables)?;
        let active = is_activity_active(activity, now, zone_offset);
        let day = current_activity_day(activity, now, zone_offset);
        let changed = self.sync_activity_boss(aid, day);
        Ok((self.activity_boss_data(boss.port_id), active, day, changed))
    }

    pub fn settle_activity_boss(
        &mut self,
        aid: i32,
        damage: i64,
        role_ids: Vec<i32>,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(i32, i64), ActivityBossError> {
        if damage < 0 {
            return Err(ActivityBossError::NegativeDamage);
        }
        let (activity, boss) = require_activity_boss_config(aid, tables)?;
        if !is_activity_active(activity, now, zone_offset) {
            return Err(ActivityBossError::Inactive(aid));
        }
        if let Some(role_id) = role_ids.iter().find(|role_id| !self.owns_role(**role_id)) {
            return Err(ActivityBossError::UnknownRole(*role_id));
        }
        let day = current_activity_day(activity, now, zone_offset);
        self.sync_activity_boss(aid, day);
        let score = (damage as f64 * boss.damage_ratio) as i64;
        if score > self.activity_boss.damage {
            self.activity_boss.damage = score;
            self.activity_boss.role_ids = role_ids;
        }
        self.activity_boss.daily_damage = self.activity_boss.daily_damage.max(score);
        if let Some(reward) = tables.activity_boss_daily_points.iter().find(|reward| {
            reward.id == day && reward.daily_point <= self.activity_boss.daily_damage
        }) {
            self.activity_boss
                .daily_rewards
                .entry(reward.id)
                .or_insert(0);
        }
        let gameplay_id = tables
            .activity_boss_ports
            .get(boss.port_id)
            .map_or(self.gameplay_id, |port| port.port_id);
        Ok((gameplay_id, self.activity_boss.damage))
    }

    pub fn claim_activity_boss_reward(
        &mut self,
        aid: i32,
        reward_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, ActivityBossError> {
        require_activity_boss_config(aid, tables)?;
        let row = tables
            .activity_boss_points
            .iter()
            .find(|row| row.id == reward_id)
            .ok_or(ActivityBossError::UnknownReward(reward_id))?;
        if self.activity_boss.aid != aid || self.activity_boss.damage < row.boss_point {
            return Err(ActivityBossError::RewardNotReached(reward_id));
        }
        if self.activity_boss.rewards.contains(&reward_id) {
            return Err(ActivityBossError::RewardAlreadyClaimed(reward_id));
        }
        let mut result = DcNetDataTakeRewardRes::default();
        self.add_reward(row.reward.key, row.reward.value, tables, now, &mut result);
        self.activity_boss.rewards.push(reward_id);
        self.activity_boss.rewards.sort_unstable();
        Ok(result)
    }

    pub fn claim_activity_boss_daily_rewards(
        &mut self,
        aid: i32,
        ids: &[i32],
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, ActivityBossError> {
        require_activity_boss_config(aid, tables)?;
        let mut rows = Vec::with_capacity(ids.len());
        for id in ids {
            let row = tables
                .activity_boss_daily_points
                .iter()
                .find(|row| row.id == *id)
                .ok_or(ActivityBossError::UnknownReward(*id))?;
            match self.activity_boss.daily_rewards.get(id) {
                Some(0) => rows.push(row),
                Some(_) => return Err(ActivityBossError::RewardAlreadyClaimed(*id)),
                None => return Err(ActivityBossError::RewardNotReached(*id)),
            }
        }
        let mut result = DcNetDataTakeRewardRes::default();
        for row in rows {
            self.add_reward(row.reward.key, row.reward.value, tables, now, &mut result);
            self.activity_boss.daily_rewards.insert(row.id, 1);
        }
        Ok(result)
    }

    pub fn activity_boss_rank(&self) -> (Vec<DcNetDataBossRankElem>, u32, Vec<i32>) {
        let ranked = self.activity_boss.damage > 0;
        let ranks = ranked
            .then(|| DcNetDataBossRankElem {
                rank: 1,
                score: self.activity_boss.damage as f64,
                name: self.nickname.clone(),
                rids: self.activity_boss.role_ids.clone(),
            })
            .into_iter()
            .collect();
        (
            ranks,
            u32::from(ranked),
            self.activity_boss.role_ids.clone(),
        )
    }

    fn sync_activity_boss(&mut self, aid: i32, day: i32) -> bool {
        if self.activity_boss.aid != aid {
            self.activity_boss = ActivityBossState {
                aid,
                day,
                ..Default::default()
            };
            return true;
        }
        if self.activity_boss.day != day {
            self.activity_boss.day = day;
            self.activity_boss.daily_damage = 0;
            return true;
        }
        false
    }

    fn activity_boss_data(&self, gameplay_id: i32) -> DcNetDataActivityBossInfo {
        DcNetDataActivityBossInfo {
            g_id: gameplay_id,
            damage: clamp_to_i32(self.activity_boss.damage),
            ids: self.activity_boss.rewards.clone(),
            daily_rewards: self
                .activity_boss
                .daily_rewards
                .iter()
                .map(|(&id, &state)| (id, state))
                .collect(),
            daily_damage: clamp_to_i32(self.activity_boss.daily_damage),
        }
    }
}

fn require_activity_boss_config(
    aid: i32,
    tables: &GameTables,
) -> Result<(&configs::tables::Activity, &configs::tables::ActivityBoss), ActivityBossError> {
    let activity = tables
        .activities
        .get(aid)
        .filter(|activity| activity.activity_type == 4)
        .ok_or(ActivityBossError::Unknown(aid))?;
    let boss = tables
        .activity_bosses
        .rows
        .first()
        .ok_or(ActivityBossError::Unknown(aid))?;
    Ok((activity, boss))
}

fn is_activity_active(activity: &configs::tables::Activity, now: i32, zone_offset: i32) -> bool {
    common::time::table_window_active(&activity.open_time, &activity.close_time, now, zone_offset)
}

fn current_activity_day(activity: &configs::tables::Activity, now: i32, zone_offset: i32) -> i32 {
    common::time::table_time_utc(&activity.open_time, zone_offset).map_or(0, |open| {
        ((i64::from(now) - open).max(0) / 86_400 + 1).min(i64::from(i32::MAX)) as i32
    })
}

fn clamp_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
#[cfg(test)]
mod tests;
