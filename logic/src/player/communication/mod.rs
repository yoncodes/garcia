use super::*;

impl Player {
    pub fn archive_infos(&self, tables: &GameTables, now: i32) -> Vec<DcNetDataArchiveInfo> {
        self.unlocked_archive_ids(tables, now)
            .into_iter()
            .map(|id| DcNetDataArchiveInfo {
                id,
                in_time: self
                    .archive_unlocks
                    .get(&id)
                    .copied()
                    .unwrap_or(self.created_at),
            })
            .collect()
    }

    pub(super) fn record_current_archive_unlocks(&mut self, tables: &GameTables, now: i32) {
        for id in self.unlocked_archive_ids(tables, now) {
            self.archive_unlocks.entry(id).or_insert(now);
        }
    }

    pub(super) fn unlocked_archive_ids(&self, tables: &GameTables, now: i32) -> Vec<i32> {
        tables
            .archives
            .rows
            .iter()
            .filter(|archive| self.is_archive_unlocked(archive, tables, now))
            .map(|archive| archive.id)
            .collect()
    }

    pub(super) fn newly_unlocked_archives_since(
        &mut self,
        previous: &[i32],
        tables: &GameTables,
        now: i32,
    ) -> Vec<DcNetDataArchiveInfo> {
        let archives = self
            .unlocked_archive_ids(tables, now)
            .into_iter()
            .filter(|id| !previous.contains(id))
            .map(|id| DcNetDataArchiveInfo { id, in_time: now })
            .collect::<Vec<_>>();
        for archive in &archives {
            self.archive_unlocks.entry(archive.id).or_insert(now);
        }
        archives
    }

    fn is_archive_unlocked(
        &self,
        archive: &configs::tables::Archive,
        tables: &GameTables,
        now: i32,
    ) -> bool {
        let conditions_met = || {
            !archive.conditions.is_empty()
                && archive.conditions.iter().all(|condition| {
                    self.condition_progress(condition.key, &condition.value, tables, now)
                        >= condition_total(condition.key, &condition.value)
                })
        };
        match archive.archive_type {
            1 => self.owns_role(archive.detail_id),
            2 => self
                .partners
                .iter()
                .any(|partner| partner.partner_id == archive.detail_id),
            3 | 7 => conditions_met(),
            4 => self
                .equips
                .iter()
                .any(|equipment| equipment.equip_id == archive.detail_id),
            _ => false,
        }
    }

    pub fn newly_unlocked_interaction_archives(
        &mut self,
        object_id: &str,
        tables: &GameTables,
        now: i32,
    ) -> Vec<DcNetDataArchiveInfo> {
        let first_completion = self
            .interact_objs
            .iter()
            .any(|object| object.object_id == object_id && object.status != 0 && object.count == 1);
        if !first_completion {
            return Vec::new();
        }
        let archives = tables
            .archives
            .rows
            .iter()
            .filter(|archive| archive.archive_type == 7)
            .filter(|archive| {
                archive
                    .conditions
                    .iter()
                    .any(|condition| condition.key == 85 && condition.value == object_id)
            })
            .filter(|archive| {
                archive.conditions.iter().all(|condition| {
                    self.condition_progress(condition.key, &condition.value, tables, now)
                        >= condition_total(condition.key, &condition.value)
                })
            })
            .map(|archive| DcNetDataArchiveInfo {
                id: archive.id,
                in_time: now,
            })
            .collect::<Vec<_>>();
        for archive in &archives {
            self.archive_unlocks.entry(archive.id).or_insert(now);
        }
        archives
    }

    pub fn mails(&self, now: i32) -> Vec<DcNetDataEmail> {
        self.mails
            .iter()
            .filter(|state| is_mail_active(state, now))
            .map(|state| {
                let mut mail = state.mail.clone();
                mail.expire = if state.expires_at == 0 {
                    0
                } else {
                    common::time::seconds_until(i64::from(state.expires_at), i64::from(now))
                };
                mail
            })
            .collect()
    }

    pub fn read_mails(&mut self, ids: &[i64], now: i32) -> (Vec<i64>, bool) {
        let mut accepted = Vec::new();
        let mut changed = false;
        for id in ids {
            let Some(state) = self
                .mails
                .iter_mut()
                .find(|state| state.mail.email_id == *id && is_mail_active(state, now))
            else {
                continue;
            };
            accepted.push(*id);
            if state.mail.is_read != 1 {
                state.mail.is_read = 1;
                changed = true;
            }
        }
        (accepted, changed)
    }

    pub fn claim_mail_attachments(
        &mut self,
        ids: &[i64],
        tables: &GameTables,
        now: i32,
    ) -> (DcNetDataTakeRewardRes, Vec<i64>, bool) {
        let mut accepted = Vec::new();
        let mut gifts = BTreeMap::<i32, i32>::new();
        for id in ids {
            let Some(state) = self.mails.iter_mut().find(|state| {
                state.mail.email_id == *id
                    && state.mail.taken != 1
                    && !state.mail.gift_list.is_empty()
                    && is_mail_active(state, now)
            }) else {
                continue;
            };
            state.mail.is_read = 1;
            state.mail.taken = 1;
            accepted.push(*id);
            for gift in &state.mail.gift_list {
                let amount = gifts.entry(gift.reward).or_default();
                *amount = amount.saturating_add(gift.amount);
            }
        }
        let mut rewards = DcNetDataTakeRewardRes::default();
        for (id, amount) in gifts {
            self.add_reward(id, amount, tables, now, &mut rewards);
        }
        let changed = !accepted.is_empty();
        (rewards, accepted, changed)
    }

    pub fn delete_mails(&mut self, ids: &[i64]) -> Vec<i64> {
        let accepted: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| {
                self.mails.iter().any(|state| {
                    state.mail.email_id == *id
                        && if state.mail.gift_list.is_empty() {
                            state.mail.is_read == 1
                        } else {
                            state.mail.taken == 1
                        }
                })
            })
            .collect();
        self.mails
            .retain(|state| !accepted.contains(&state.mail.email_id));
        accepted
    }

    pub fn red_dots(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataRedDot> {
        self.red_dot_candidates(tables, now, zone_offset)
            .into_iter()
            .filter(|red_dot| !self.checked_red_dots.contains(&red_dot.id))
            .collect()
    }

    pub fn reward_item_red_dots(
        &self,
        reward: &DcNetDataTakeRewardRes,
        tables: &GameTables,
    ) -> Vec<DcNetDataRedDot> {
        let mut red_dots = Vec::new();
        for item in &reward.items {
            if tables
                .items
                .get(item.item_id)
                .is_some_and(|definition| definition.item_type == 103)
            {
                push_red_dot(&mut red_dots, Reddot::Item, item.item_id);
            }
        }
        red_dots
    }

    pub fn check_red_dots(
        &mut self,
        ids: &[String],
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> (Vec<DcNetDataRedDot>, bool) {
        let candidates = self.red_dot_candidates(tables, now, zone_offset);
        let mut changed = false;
        let reddots = ids
            .iter()
            .filter_map(|id| {
                candidates
                    .iter()
                    .find(|red_dot| red_dot.id == *id)
                    .cloned()
                    .or_else(|| red_dot_from_id(id))
            })
            .map(|mut red_dot| {
                red_dot.is_checked = true;
                if !self.checked_red_dots.contains(&red_dot.id) {
                    self.checked_red_dots.push(red_dot.id.clone());
                    changed = true;
                }
                red_dot
            })
            .collect();
        (reddots, changed)
    }

    fn red_dot_candidates(
        &self,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Vec<DcNetDataRedDot> {
        let mut reddots = Vec::new();
        for feature in &self.feats {
            if feature.status == FeatStatus::FeatUnlocked as i32 {
                push_red_dot(&mut reddots, Reddot::Feat, feature.id);
            }
        }
        for gacha in self.gacha_pools(tables, now, zone_offset) {
            push_red_dot(&mut reddots, Reddot::Gacha, gacha.gacha_id);
        }
        for avatar in &self.profile_avatars {
            push_red_dot(&mut reddots, Reddot::Avatar, avatar.id);
        }
        for frame in &self.profile_frames {
            push_red_dot(&mut reddots, Reddot::Frame, frame.id);
        }
        for card in &self.profile_cards {
            push_red_dot(&mut reddots, Reddot::Card, card.id);
        }
        for title in &self.profile_titles {
            push_red_dot(&mut reddots, Reddot::Title, title.id);
        }
        for album in &self.albums {
            if album.status == AlbumStatus::Unlocked as i32 {
                push_red_dot(&mut reddots, Reddot::Album, album.id);
            }
        }
        for archive in &tables.archives.rows {
            if self.is_archive_unlocked(archive, tables, now) {
                push_red_dot(&mut reddots, Reddot::Archive, archive.id);
            }
        }
        reddots
    }
}

fn red_dot_from_id(id: &str) -> Option<DcNetDataRedDot> {
    let (kind, arg) = id.split_once('_')?;
    let func_id = kind.parse::<i32>().ok()?;
    Reddot::try_from(func_id).ok()?;
    (!arg.is_empty()).then(|| DcNetDataRedDot {
        arg: arg.to_owned(),
        func_id,
        id: id.to_owned(),
        ..Default::default()
    })
}
#[cfg(test)]
mod tests;
