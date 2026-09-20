use super::super::*;

mod activities;
mod collection;
mod communication;
mod equipment;
mod inventory;
mod profile;
mod progression;
mod world;

impl Player {
    pub fn to_record(&self) -> database::models::game::player_state::PlayerRecord {
        let mut record = database::models::game::player_state::PlayerRecord {
            uid: self.uid,
            username: self.username.clone(),
            nickname: self.nickname.clone(),
            level: self.level,
            gameplay_id: self.gameplay_id,
            created_at: self.created_at,
            last_login_time: self.last_login_time,
            cur_form: self.cur_form,
            profile_avatar: self.profile_avatar,
            profile_card: self.profile_card,
            profile_title: self.profile_title,
            profile_frame: self.profile_frame,
            banner_girl: self.banner_girl,
            traced_task_group: self.traced_task_group,
            heat_updated_at: self.heat_updated_at,
            region: self.region,
            ..Default::default()
        };

        profile::write(self, &mut record);
        inventory::write(self, &mut record);
        equipment::write(self, &mut record);
        collection::write(self, &mut record);
        world::write(self, &mut record);
        progression::write(self, &mut record);
        activities::write(self, &mut record);
        communication::write(self, &mut record);
        record
    }
}
