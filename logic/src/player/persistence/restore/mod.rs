use super::super::*;

mod activities;
mod collection;
mod communication;
mod equipment;
mod exploration;
mod inventory;
mod profile;
mod progression;
mod world;

impl Player {
    pub fn from_record(
        mut record: database::models::game::player_state::PlayerRecord,
        tables: &GameTables,
        defaults: &AccountDefaults,
    ) -> Self {
        let mut player = Self::new(record.uid, tables, defaults);
        player.username = std::mem::take(&mut record.username);
        player.nickname = std::mem::take(&mut record.nickname);
        if player.nickname == format!("Player{}", record.uid) {
            player.nickname.clear();
        }
        player.level = std::mem::take(&mut record.level);
        player.gameplay_id = std::mem::take(&mut record.gameplay_id);
        player.created_at = std::mem::take(&mut record.created_at);
        player.last_login_time = std::mem::take(&mut record.last_login_time);
        player.cur_form = std::mem::take(&mut record.cur_form);
        player.profile_avatar = std::mem::take(&mut record.profile_avatar);
        player.profile_card = std::mem::take(&mut record.profile_card);
        player.profile_title = std::mem::take(&mut record.profile_title);
        player.profile_frame = std::mem::take(&mut record.profile_frame);
        player.banner_girl = std::mem::take(&mut record.banner_girl);
        player.traced_task_group = std::mem::take(&mut record.traced_task_group);
        player.heat_updated_at = record.heat_updated_at.max(player.created_at);
        profile::restore(&mut player, &mut record);
        inventory::restore(&mut player, &mut record);
        equipment::restore(&mut player, &mut record, tables);
        collection::restore(&mut player, &mut record);
        world::restore(&mut player, &mut record, tables);
        progression::restore(&mut player, &mut record, tables);
        let owned_role_ids = player
            .roles
            .iter()
            .filter_map(|role| role.role_basic_info.as_ref().map(|role| role.game_role_id))
            .collect::<Vec<_>>();
        for maid_id in owned_role_ids {
            player.unlock_role_avatar(maid_id, tables);
            player.unlock_maxed_role_namecard(maid_id, tables);
        }
        exploration::restore(&mut player, &mut record, tables);
        activities::restore(&mut player, &mut record, tables);
        communication::restore(&mut player, &mut record);
        player.advance_features(tables);
        player
            .record_current_archive_unlocks(tables, player.last_login_time.max(player.created_at));
        player
    }
}
