use super::*;

impl Player {
    pub fn new(uid: i64, tables: &GameTables, defaults: &AccountDefaults) -> Self {
        let now = ServerTime::now_seconds_i32();
        let daily_day = common::time::day_index(now);
        let daily_tasks = initial_daily_tasks(uid, daily_day, tables);
        let mut maid_ids = vec![tables.default_main_maid()];
        maid_ids.extend(tables.cultivation_constants.initial_maid.iter().copied());
        maid_ids.dedup();

        let roles = maid_ids
            .iter()
            .filter_map(|id| starter_role(uid, *id, tables, defaults))
            .collect();
        let profile_avatars = maid_ids
            .iter()
            .filter_map(|id| tables.profile_avatar_for_maid(*id))
            .map(|avatar| DcNetDataProfileAvatar { id: avatar.id })
            .collect();
        let team: Vec<DcNetDataFormationPos> = maid_ids
            .iter()
            .filter(|id| {
                **id == tables.default_main_maid()
                    || tables.maids.get(**id).is_some_and(|maid| maid.in_team != 0)
            })
            .map(|id| DcNetDataFormationPos {
                game_role_id: *id,
                key_num: defaults.formation_key,
            })
            .take(tables.cultivation_constants.formation_member_limit as usize)
            .collect();
        // Formation 0 is the client's preset team. TEAM_ROUTINE_NUM counts the
        // normal teams after it, so the official bootstrap contains 0..=5.
        let formations = (0..=tables.cultivation_constants.fixed_formation_count)
            .map(|formation_id| DcNetDataFormation {
                formation_id,
                remark: String::new(),
                poss: if formation_id == defaults.active_formation {
                    team.clone()
                } else {
                    Vec::new()
                },
            })
            .collect();
        let missions = tables
            .missions
            .rows
            .iter()
            .filter(|mission| mission.mission_type == 1 && mission.finish_limit.len() == 1)
            .filter_map(|mission| {
                let limit = &mission.finish_limit[0];
                let total_num = limit.value.parse::<i32>().ok()?;
                let completed = defaults.level >= total_num;
                Some(DcNetDataMissonInfo {
                    misson_id: mission.id,
                    total_num,
                    curr_num: if limit.key == 5 { defaults.level } else { 0 },
                    r#type: mission.mission_type,
                    created_at: if completed { now } else { 0 },
                    ..Default::default()
                })
            })
            .collect();
        let tasks = tables
            .initial_task_ids()
            .map(|id| DcNetDataTaskStatus {
                id,
                status: TaskStatus::Picked as i32,
                picked_at: i64::from(now),
                ..Default::default()
            })
            .collect();
        let mut resources: Vec<_> = tables
            .cultivation_constants
            .initial_resource
            .iter()
            .map(|entry| (entry.key, entry.value))
            .collect();
        resources.push((
            tables.cultivation_constants.ticket_item_id,
            tables.cultivation_constants.ticket_initial,
        ));
        let items = resources
            .into_iter()
            .filter_map(|(id, amount)| {
                tables.items.get(id).map(|item| DcNetDataItem {
                    user_item_id: make_entity_id(uid, id),
                    item_id: id,
                    amount,
                    remain_sec: 0,
                    quality: item.quality,
                })
            })
            .collect();
        let profile_avatar = tables
            .profile_avatar_for_maid(tables.default_main_maid())
            .map_or(0, |avatar| avatar.id);
        let interact_objs = tables
            .interaction_limits
            .rows
            .iter()
            .filter(|object| !object.interactob_limit.is_empty())
            .map(|object| DcNetDataInteractObj {
                object_id: object.interact_id.clone(),
                ..Default::default()
            })
            .collect();
        let challenges = tables
            .city_challenges
            .rows
            .iter()
            .map(|challenge| DcNetDataChallenge {
                id: challenge.city_challenge_id.clone(),
                ..Default::default()
            })
            .collect();
        let mut albums: Vec<_> = tables
            .albums
            .rows
            .iter()
            .filter(|album| album.conditions.is_empty())
            .map(|album| DcNetDataAlbums {
                id: album.id,
                status: AlbumStatus::Unlocked as i32,
                sort: 0,
            })
            .collect();
        if let Some(album) = albums.first_mut() {
            album.status = AlbumStatus::Selected as i32;
        }
        let (dense_fogs, locals) = tables.initial_location().map_or_else(
            || (Vec::new(), Vec::new()),
            |(map_id, local)| {
                (
                    vec![map_id],
                    vec![DcNetDataLocal {
                        region: 0,
                        local,
                        local2: String::new(),
                    }],
                )
            },
        );

        let mut player = Self {
            uid,
            username: uid.to_string(),
            nickname: String::new(),
            level: defaults.level,
            gameplay_id: defaults.gameplay_id,
            created_at: now,
            last_login_time: now,
            cur_form: defaults.active_formation,
            profile_avatar,
            profile_card: tables.cultivation_constants.initial_profile_card,
            profile_title: tables.cultivation_constants.initial_profile_title,
            profile_frame: tables.cultivation_constants.initial_profile_frame,
            banner_girl: tables.default_main_maid(),
            traced_task_group: 0,
            heat_updated_at: now,
            items,
            item_acquired: HashMap::new(),
            item_spent: HashMap::new(),
            roles,
            formations,
            missions,
            tasks,
            ports: Vec::new(),
            completed_dungeons: Vec::new(),
            dungeon_clears: HashMap::new(),
            equips: Vec::new(),
            equipment_word_plans: HashMap::new(),
            equipment_groups: Vec::new(),
            team_cores: Vec::new(),
            team_equips: Vec::new(),
            skillstones: Vec::new(),
            partners: Vec::new(),
            feats: Vec::new(),
            skins: Vec::new(),
            monster_manuals: Vec::new(),
            interact_objs,
            challenges,
            albums,
            collections: Vec::new(),
            collection_suit_rewards: Vec::new(),
            collection_places: Vec::new(),
            tp_map: BTreeMap::new(),
            role_attrs: Vec::new(),
            playing_port: None,
            version_task_claims: Vec::new(),
            dense_fogs,
            profile_frames: vec![DcNetDataProfileFrame {
                id: tables.cultivation_constants.initial_profile_frame,
                end_time: defaults.permanent_profile_end_time,
            }],
            profile_avatars,
            profile_cards: vec![DcNetDataProfileCard {
                id: tables.cultivation_constants.initial_profile_card,
            }],
            profile_titles: vec![DcNetDataProfileTitle {
                id: tables.cultivation_constants.initial_profile_title,
                end_time: defaults.permanent_profile_end_time,
                num: 0,
            }],
            city_guides: Vec::new(),
            user_guides: Vec::new(),
            region: 0,
            locals,
            favors: Vec::new(),
            favor_day: i32::MIN,
            favor_touches: 0,
            sms: Vec::new(),
            daily_day,
            daily_activity: 0,
            daily_reward_progress: 0,
            heat_exchange_day: daily_day,
            heat_exchange_count: 0,
            daily_tasks,
            gold_coins: Vec::new(),
            collection_resources: Vec::new(),
            wild_monster_day: daily_day,
            defeated_monster_points: Vec::new(),
            region_coin_daily: HashMap::new(),
            region_ticket_day: i32::MIN,
            region_levels: HashMap::new(),
            reward_boxes: Vec::new(),
            achievements: Vec::new(),
            activity_7day_claims: Vec::new(),
            activity_7day_point_claims: Vec::new(),
            activity_level_rewards: Vec::new(),
            activity_limited_level_rewards: Vec::new(),
            activity_challenge_point_claims: Vec::new(),
            version_challenges: Vec::new(),
            more_team_challenges: Vec::new(),
            gachas: Vec::new(),
            gacha_logs: Vec::new(),
            checkins: Vec::new(),
            checked_red_dots: Vec::new(),
            mails: Vec::new(),
            battle_pass: BattlePassState::default(),
            mall_purchases: Vec::new(),
            recharge_purchases: Vec::new(),
            ship_tags: Vec::new(),
            archive_unlocks: HashMap::new(),
            boss_rush: BossRushState::default(),
            activity_boss: ActivityBossState::default(),
            rift: RiftState::default(),
            rouge_progress: HashMap::new(),
            rouge_technology: Vec::new(),
            rouge_score: 0,
            rouge_score_point_at: 0,
            rouge_run: None,
        };
        player.record_current_archive_unlocks(tables, now);
        player
    }
}
#[cfg(test)]
mod tests;
