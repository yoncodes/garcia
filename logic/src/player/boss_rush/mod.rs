use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BossRushState {
    pub id: i32,
    pub ports: Vec<DcNetDataBossRushBase>,
    pub rewards: Vec<i32>,
    pub ranking_rewards: Vec<i32>,
}

impl Player {
    pub fn sync_boss_rush(&mut self, id: i32) -> bool {
        if self.boss_rush.id == id {
            return false;
        }
        let ranking_rewards = std::mem::take(&mut self.boss_rush.ranking_rewards);
        self.boss_rush = BossRushState {
            id,
            ranking_rewards,
            ..Default::default()
        };
        true
    }

    pub fn boss_rush_score(&self) -> i64 {
        self.boss_rush.ports.iter().map(|port| port.damage).sum()
    }

    pub fn deliver_boss_rush_ranking_reward(
        &mut self,
        bid: i32,
        rank: i32,
        participant_count: i32,
        tables: &GameTables,
        now: i32,
    ) -> Option<DcNetDataEmail> {
        if self.boss_rush.id != bid
            || self.boss_rush_score() == 0
            || self.boss_rush.ranking_rewards.contains(&bid)
        {
            return None;
        }

        let percentile = (rank.saturating_sub(1) * 100) / participant_count.max(1);
        let reward = tables.boss_rush_ranking_rewards.iter().find(|reward| {
            if reward.bossrush_id != bid || reward.upper_limit.key != reward.lower_limit.key {
                return false;
            }
            let value = match reward.upper_limit.key {
                1 => rank,
                2 => percentile,
                _ => return false,
            };
            value >= reward.upper_limit.value
                && (value < reward.lower_limit.value
                    || reward.lower_limit.value == 100 && value == 100)
        })?;

        let mail = DcNetDataEmail {
            email_id: 8_000_000_000_000_i64 + i64::from(bid),
            sent_at: now,
            sender: 2,
            gift_list: reward
                .reward
                .iter()
                .map(|item| DcNetDataReward {
                    reward: item.key,
                    reward_type: 2,
                    amount: item.value,
                })
                .collect(),
            sys_mail_id: 5,
            paramter: rank.to_string(),
            expire: 30 * 24 * 60 * 60,
            ..Default::default()
        };
        self.mails.push(MailState {
            mail: mail.clone(),
            expires_at: now.saturating_add(30 * 24 * 60 * 60),
        });
        self.boss_rush.ranking_rewards.push(bid);
        self.boss_rush.ranking_rewards.sort_unstable();
        Some(mail)
    }

    pub fn reset_boss_rush_team(
        &mut self,
        event: &configs::tables::BossRush,
        pid: i32,
    ) -> Result<Vec<DcNetDataBossRushBase>, BossRushError> {
        if !event.port_id.contains(&pid) {
            return Err(BossRushError::UnknownPort(pid));
        }
        self.sync_boss_rush(event.id);
        self.boss_rush.ports.retain(|port| port.pid != pid);
        Ok(self.boss_rush.ports.clone())
    }

    pub fn settle_boss_rush(
        &mut self,
        event: &configs::tables::BossRush,
        pid: i32,
        damage: i64,
        role_ids: Vec<i32>,
    ) -> Result<(i64, Vec<DcNetDataBossRushBase>), BossRushError> {
        if damage < 0 {
            return Err(BossRushError::NegativeDamage);
        }
        if !event.port_id.contains(&pid) {
            return Err(BossRushError::UnknownPort(pid));
        }
        if let Some(role_id) = role_ids.iter().find(|role_id| !self.owns_role(**role_id)) {
            return Err(BossRushError::UnknownRole(*role_id));
        }
        self.sync_boss_rush(event.id);
        if let Some(port) = self.boss_rush.ports.iter_mut().find(|port| port.pid == pid) {
            if damage > port.damage {
                port.damage = damage;
                port.role_ids = role_ids;
            }
        } else {
            self.boss_rush.ports.push(DcNetDataBossRushBase {
                pid,
                role_ids,
                damage,
            });
        }
        self.boss_rush.ports.sort_by_key(|port| port.pid);
        Ok((self.boss_rush_score(), self.boss_rush.ports.clone()))
    }

    pub fn claim_boss_rush_rewards(
        &mut self,
        bid: i32,
        sid: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataTakeRewardRes, Vec<i32>), BossRushError> {
        if self.boss_rush.id != bid {
            return Err(BossRushError::Inactive(bid));
        }
        let score = self.boss_rush_score();
        let rows: Vec<_> = tables
            .boss_rush_rewards
            .iter()
            .filter(|reward| reward.bossrush_id == bid && (sid == 0 || reward.id == sid))
            .cloned()
            .collect();
        if sid != 0 && rows.is_empty() {
            return Err(BossRushError::UnknownReward(sid));
        }
        if sid != 0 && self.boss_rush.rewards.contains(&sid) {
            return Err(BossRushError::RewardAlreadyClaimed(sid));
        }
        if sid != 0 && rows[0].boss_point > score {
            return Err(BossRushError::RewardNotReached(sid));
        }

        let mut result = DcNetDataTakeRewardRes::default();
        for row in rows {
            if row.boss_point > score || self.boss_rush.rewards.contains(&row.id) {
                continue;
            }
            for reward in row.reward {
                self.add_reward(reward.key, reward.value, tables, now, &mut result);
            }
            self.boss_rush.rewards.push(row.id);
        }
        self.boss_rush.rewards.sort_unstable();
        Ok((result, self.boss_rush.rewards.clone()))
    }
}
#[cfg(test)]
mod tests;
