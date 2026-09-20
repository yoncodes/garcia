use super::*;

impl Player {
    pub fn unlock_all_teleport_gates(&mut self, tables: &GameTables) -> Vec<DcNetDataInteractObj> {
        let mut updates = Vec::new();
        for anchor in &tables.teleportation_anchors.rows {
            let object = if let Some(object) = self
                .interact_objs
                .iter_mut()
                .find(|object| object.object_id == anchor.id)
            {
                object
            } else {
                self.interact_objs.push(DcNetDataInteractObj {
                    object_id: anchor.id.clone(),
                    ..Default::default()
                });
                self.interact_objs.last_mut().unwrap()
            };
            if object.status <= 0 {
                object.status = 1;
                object.interactive = true;
                updates.push(object.clone());
            }
        }
        updates
    }

    pub fn refresh_interactions(&mut self, tables: &GameTables) -> Vec<DcNetDataInteractObj> {
        let unlocked = tables
            .interaction_limits
            .rows
            .iter()
            .filter(|config| !config.interactob_limit.is_empty())
            .filter(|config| {
                config.interactob_limit.iter().all(|limit| match limit.key {
                    50 | 81 | 85 => {
                        self.condition_progress(
                            limit.key,
                            &limit.value,
                            tables,
                            ServerTime::now_seconds_i32(),
                        ) >= condition_total(limit.key, &limit.value)
                    }
                    83 => limit
                        .value
                        .rsplit_once('|')
                        .and_then(|(object_id, status)| {
                            status.parse::<i32>().ok().map(|status| (object_id, status))
                        })
                        .is_some_and(|(object_id, status)| {
                            self.interact_objs.iter().any(|object| {
                                object.object_id == object_id && object.status == status
                            })
                        }),
                    _ => false,
                })
            })
            .map(|config| config.interact_id.clone())
            .collect::<Vec<_>>();

        let mut updates = Vec::new();
        for object_id in unlocked {
            let object = if let Some(object) = self
                .interact_objs
                .iter_mut()
                .find(|object| object.object_id == object_id)
            {
                object
            } else {
                self.interact_objs.push(DcNetDataInteractObj {
                    object_id,
                    ..Default::default()
                });
                self.interact_objs.last_mut().unwrap()
            };
            if !object.interactive {
                object.interactive = true;
                updates.push(object.clone());
            }
        }
        updates
    }

    pub fn add_formation(
        &mut self,
        tables: &GameTables,
    ) -> Result<DcNetDataFormation, FormationError> {
        let first = tables
            .cultivation_constants
            .fixed_formation_count
            .checked_add(1)
            .ok_or(FormationError::InvalidConfig)?;
        let end = first
            .checked_add(tables.cultivation_constants.custom_formation_count)
            .filter(|end| first >= 0 && *end >= first)
            .ok_or(FormationError::InvalidConfig)?;
        let formation_id = (first..end)
            .find(|id| {
                !self
                    .formations
                    .iter()
                    .any(|formation| formation.formation_id == *id)
            })
            .ok_or(FormationError::LimitReached)?;
        let formation = DcNetDataFormation {
            formation_id,
            ..Default::default()
        };
        self.formations.push(formation.clone());
        Ok(formation)
    }

    pub fn delete_formation(
        &mut self,
        formation_id: i32,
        tables: &GameTables,
    ) -> Result<(), FormationError> {
        if formation_id <= tables.cultivation_constants.fixed_formation_count {
            return Err(FormationError::Fixed(formation_id));
        }
        if formation_id == self.cur_form {
            return Err(FormationError::Current(formation_id));
        }
        let index = self
            .formations
            .iter()
            .position(|formation| formation.formation_id == formation_id)
            .ok_or(FormationError::Unknown(formation_id))?;
        self.formations.remove(index);
        Ok(())
    }

    pub fn set_tp_map(&mut self, key: String, value: String) -> bool {
        if self
            .tp_map
            .get(&key)
            .is_some_and(|current| current == &value)
        {
            return false;
        }
        self.tp_map.insert(key, value);
        true
    }

    pub fn set_playing_port(&mut self, port_index_id: i32, port_info: String) -> bool {
        let next = PlayingPortState {
            port_index_id,
            port_info,
        };
        if self.playing_port.as_ref() == Some(&next) {
            return false;
        }
        self.playing_port = Some(next);
        true
    }

    pub fn world_level(&self, tables: &GameTables) -> i32 {
        tables
            .world_levels
            .iter()
            .filter(|level| {
                self.tasks
                    .iter()
                    .any(|task| task.id == level.task_id && task.status == TaskStatus::Done as i32)
            })
            .map(|level| level.level)
            .max()
            .unwrap_or(0)
    }

    pub fn record_city_guide(
        &mut self,
        id: String,
        tables: &GameTables,
    ) -> Result<bool, CityGuideError> {
        if tables.city_guides.get(&id).is_none() {
            return Err(CityGuideError(id));
        }
        if !self.city_guides.contains(&id) {
            self.city_guides.push(id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn record_user_guide(&mut self, gid: String) -> bool {
        if !self.user_guides.contains(&gid) {
            self.user_guides.push(gid);
            true
        } else {
            false
        }
    }

    pub fn complete_all_tutorials(&mut self, tables: &GameTables) -> usize {
        tables
            .tutorials
            .rows
            .iter()
            .filter(|tutorial| self.record_user_guide(tutorial.group_id.to_string()))
            .count()
    }

    pub fn reset_tutorials(&mut self, tables: &GameTables) -> usize {
        let groups = tables
            .tutorials
            .rows
            .iter()
            .map(|tutorial| tutorial.group_id.to_string())
            .collect::<std::collections::HashSet<_>>();
        let before = self.user_guides.len();
        self.user_guides.retain(|gid| !groups.contains(gid));
        before - self.user_guides.len()
    }

    pub fn set_local(&mut self, local: DcNetDataLocal) -> bool {
        self.region = local.region;
        if let Some(existing) = self
            .locals
            .iter_mut()
            .find(|existing| existing.region == local.region)
        {
            if *existing == local {
                return false;
            }
            *existing = local;
        } else {
            self.locals.push(local);
        }
        true
    }

    pub fn refresh_daily_world(&mut self, now: i32) -> bool {
        let day = common::time::day_index(now);
        if self.wild_monster_day == day {
            return false;
        }
        self.wild_monster_day = day;
        self.defeated_monster_points.clear();
        self.region_coin_daily.clear();
        true
    }

    pub(super) fn monster_point_group(
        &self,
        group_id: &str,
        tables: &GameTables,
    ) -> DcNetDataMonsterPointGroup {
        DcNetDataMonsterPointGroup {
            id: group_id.to_owned(),
            points: tables
                .main_city_monster_points
                .rows
                .iter()
                .filter(|point| point.group_id == group_id)
                .filter(|point| !self.defeated_monster_points.contains(&point.id))
                .map(|point| point.id.clone())
                .collect(),
        }
    }

    pub fn set_formation(
        &mut self,
        formation: DcNetDataFormation,
        tables: &GameTables,
    ) -> Result<bool, FormationError> {
        let maximum = tables.cultivation_constants.formation_member_limit as usize;
        if formation.poss.len() > maximum {
            return Err(FormationError::TooManyMembers {
                formation_id: formation.formation_id,
                members: formation.poss.len(),
                maximum,
            });
        }
        if let Some(member) = formation
            .poss
            .iter()
            .find(|member| !self.owns_role(member.game_role_id))
        {
            return Err(FormationError::RoleNotOwned(member.game_role_id));
        }
        let Some(current) = self
            .formations
            .iter_mut()
            .find(|current| current.formation_id == formation.formation_id)
        else {
            return Err(FormationError::Unknown(formation.formation_id));
        };
        if *current == formation {
            return Ok(false);
        }
        *current = formation;
        Ok(true)
    }

    pub fn select_formation(&mut self, id: i32) -> Result<bool, FormationError> {
        if !self
            .formations
            .iter()
            .any(|formation| formation.formation_id == id)
        {
            return Err(FormationError::Unknown(id));
        }
        if self.cur_form == id {
            return Ok(false);
        }
        self.cur_form = id;
        Ok(true)
    }

    pub(super) fn set_interact(
        &mut self,
        object_id: &str,
        status: i32,
    ) -> Option<DcNetDataInteractObj> {
        let index = self
            .interact_objs
            .iter()
            .position(|object| object.object_id == object_id)
            .unwrap_or_else(|| {
                self.interact_objs.push(DcNetDataInteractObj {
                    object_id: object_id.to_owned(),
                    ..Default::default()
                });
                self.interact_objs.len() - 1
            });
        let object = &mut self.interact_objs[index];
        object.status = status;
        object.count = object.count.saturating_add(1);
        Some(object.clone())
    }

    pub fn complete_interaction(
        &mut self,
        object_id: &str,
        status: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<InteractionOutcome, InteractionError> {
        if !tables.is_interaction_object(object_id) {
            return Err(InteractionError::Unknown(object_id.to_owned()));
        }
        let rewards = if let Some(event) = tables
            .city_events
            .get(object_id)
            .filter(|event| event.reward_id != 0)
        {
            let claim_limit = event.interact_num.max(1);
            let stored = self
                .interact_objs
                .iter()
                .find(|object| object.object_id == object_id);
            if stored.is_some_and(|object| object.count >= claim_limit) {
                return Ok(InteractionOutcome {
                    object: stored.cloned().expect("the interaction was found above"),
                    reward: None,
                });
            }
            let bundle = tables.rewards.get(event.reward_id).ok_or_else(|| {
                InteractionError::UnsupportedReward {
                    object_id: object_id.to_owned(),
                    reward_id: event.reward_id,
                }
            })?;
            Some(bundle.reward.clone())
        } else {
            None
        };
        let object = self
            .set_interact(object_id, status)
            .expect("set_interact always returns the stored interaction");
        let reward = rewards.map(|rewards| {
            let mut result = DcNetDataTakeRewardRes::default();
            for reward in rewards {
                self.add_reward(reward.key, reward.value, tables, now, &mut result);
            }
            result
        });
        Ok(InteractionOutcome { object, reward })
    }

    pub fn record_interaction(
        &mut self,
        object_id: &str,
        status: i32,
        tables: &GameTables,
    ) -> Result<Option<DcNetDataInteractObj>, InteractionError> {
        if !tables.is_interaction_object(object_id) {
            return Err(InteractionError::Unknown(object_id.to_owned()));
        }
        Ok(self.set_interact(object_id, status))
    }

    pub fn complete_city_challenge(
        &mut self,
        id: &str,
        stars: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<(DcNetDataChallenge, Option<DcNetDataTakeRewardRes>), CityChallengeError> {
        let config = tables
            .city_challenges
            .get(id)
            .ok_or_else(|| CityChallengeError::Unknown(id.to_owned()))?;
        let challenge_index = self
            .challenges
            .iter()
            .position(|challenge| challenge.id == id)
            .ok_or_else(|| CityChallengeError::Unknown(id.to_owned()))?;

        let tier_count = i32::try_from(config.reward.len()).unwrap_or(i32::MAX);
        if stars < 0 || stars > tier_count {
            return Err(CityChallengeError::InvalidStars {
                id: id.to_owned(),
                stars,
            });
        }

        let previous_stars = self.challenges[challenge_index].stars.max(0);
        let earned_stars = previous_stars.max(stars);
        // The three server reward bundle IDs are not present in the extracted
        // reward table. Their adjacent preview columns do contain the exact
        // item/amount pairs for each star tier, so use those table values
        // instead of inventing a bundle mapping.
        let mut reward = (config.challenge_type == 2).then(DcNetDataTakeRewardRes::default);
        let mut earned_items = Vec::<(i32, i32)>::new();
        for tier in previous_stars..earned_stars {
            let preview = config
                .reward_preview(usize::try_from(tier).unwrap())
                .filter(|preview| !preview.is_empty())
                .ok_or_else(|| CityChallengeError::MissingRewardPreview {
                    id: id.to_owned(),
                    tier: tier + 1,
                })?;
            for item in preview {
                if let Some((_, amount)) = earned_items
                    .iter_mut()
                    .find(|(item_id, _)| *item_id == item.key)
                {
                    *amount += item.value;
                } else {
                    earned_items.push((item.key, item.value));
                }
            }
        }
        if let Some(result) = &mut reward {
            for (item_id, amount) in earned_items {
                self.add_reward(item_id, amount, tables, now, result);
            }
        }

        let challenge = &mut self.challenges[challenge_index];
        challenge.finished = true;
        challenge.stars = earned_stars;
        Ok((challenge.clone(), reward))
    }

    pub fn set_challenge_result(&mut self, id: &str, stars: i32) -> Option<DcNetDataChallenge> {
        let challenge = self
            .challenges
            .iter_mut()
            .find(|challenge| challenge.id == id)?;
        challenge.finished = true;
        challenge.stars = challenge.stars.max(stars);
        Some(challenge.clone())
    }

    pub fn select_albums(&mut self, ids: &[i32]) -> Result<bool, AlbumSelectionError> {
        if let Some(id) = ids
            .iter()
            .find(|id| !self.albums.iter().any(|album| album.id == **id))
        {
            return Err(AlbumSelectionError(*id));
        }
        let mut changed = false;
        for album in &mut self.albums {
            let status = if ids.contains(&album.id) {
                AlbumStatus::Selected as i32
            } else {
                AlbumStatus::Unlocked as i32
            };
            changed |= album.status != status;
            album.status = status;
        }
        Ok(changed)
    }

    pub(super) fn unlock_available_albums(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Vec<DcNetDataAlbums> {
        let mut unlocked = tables
            .albums
            .rows
            .iter()
            .filter(|config| !self.albums.iter().any(|album| album.id == config.id))
            .filter(|config| {
                config.conditions.iter().all(|condition| {
                    self.condition_progress(condition.key, &condition.value, tables, now)
                        >= condition_total(condition.key, &condition.value)
                })
            })
            .map(|config| DcNetDataAlbums {
                id: config.id,
                status: AlbumStatus::Unlocked as i32,
                sort: 0,
            })
            .collect::<Vec<_>>();

        self.albums.extend(unlocked.iter().copied());
        self.albums.sort_by_key(|album| album.id);
        unlocked.sort_by_key(|album| album.id);
        unlocked
    }

    pub fn add_dense_fogs(&mut self, ids: impl IntoIterator<Item = i32>) -> bool {
        let mut changed = false;
        for id in ids {
            if !self.dense_fogs.contains(&id) {
                self.dense_fogs.push(id);
                changed = true;
            }
        }
        changed
    }
}
#[cfg(test)]
mod tests;
