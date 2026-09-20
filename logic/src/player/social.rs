use protocol::pbcommon::{DcNetDataItem, DcNetDataSocialUserProfile};

use super::*;

impl Player {
    pub fn social_profile(
        &self,
        tables: &GameTables,
        has_gift: bool,
        gift_sent: bool,
        now: i32,
    ) -> DcNetDataSocialUserProfile {
        let roles = self
            .formations
            .iter()
            .find(|formation| formation.formation_id == self.cur_form)
            .map(|formation| {
                formation
                    .poss
                    .iter()
                    .map(|position| position.game_role_id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();
        DcNetDataSocialUserProfile {
            uid: self.uid,
            nickname: self.nickname.clone(),
            avatar: self.profile_avatar,
            avatar_frame: self.profile_frame,
            card: self.profile_card,
            title: self.profile_title,
            level: self.level,
            world_lv: self.world_level(tables),
            chest_num: self
                .reward_boxes
                .iter()
                .filter(|chest| chest.status != 0)
                .count()
                .try_into()
                .unwrap_or(i32::MAX),
            roles,
            online_time: String::new(),
            offline_time: String::new(),
            util_last_login_sec: now.saturating_sub(self.last_login_time),
            gift: i32::from(has_gift),
            gift_sent: i32::from(gift_sent),
            ..Default::default()
        }
    }

    pub fn grant_friend_gift(&mut self, tables: &GameTables) -> Vec<DcNetDataItem> {
        let rewards: Vec<_> = tables
            .cultivation_constants
            .friend_gift
            .iter()
            .map(|reward| (reward.key, reward.value))
            .collect();
        rewards
            .into_iter()
            .filter_map(|(id, amount)| self.add_item(id, amount, tables))
            .collect()
    }
}
