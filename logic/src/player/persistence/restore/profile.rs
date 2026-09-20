use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    for profile in std::mem::take(&mut record.profile_unlocks) {
        match profile.profile_type {
            1 => {
                let value = DcNetDataProfileAvatar {
                    id: profile.profile_id,
                };
                if !player
                    .profile_avatars
                    .iter()
                    .any(|profile| profile.id == value.id)
                {
                    player.profile_avatars.push(value);
                }
            }
            2 => {
                let value = DcNetDataProfileFrame {
                    id: profile.profile_id,
                    end_time: 0,
                };
                if !player
                    .profile_frames
                    .iter()
                    .any(|profile| profile.id == value.id)
                {
                    player.profile_frames.push(value);
                }
            }
            3 => {
                let value = DcNetDataProfileTitle {
                    id: profile.profile_id,
                    end_time: 0,
                    num: 1,
                };
                if !player
                    .profile_titles
                    .iter()
                    .any(|profile| profile.id == value.id)
                {
                    player.profile_titles.push(value);
                }
            }
            4 => {
                let value = DcNetDataProfileCard {
                    id: profile.profile_id,
                };
                if !player
                    .profile_cards
                    .iter()
                    .any(|profile| profile.id == value.id)
                {
                    player.profile_cards.push(value);
                }
            }
            _ => {}
        }
    }
    player.skins = std::mem::take(&mut record.skins);
    player.ship_tags = std::mem::take(&mut record.ship_tags);
    player.archive_unlocks = std::mem::take(&mut record.archive_unlocks)
        .into_iter()
        .map(|archive| (archive.archive_id, archive.unlocked_at))
        .collect();
    player.city_guides = std::mem::take(&mut record.city_guides);
    player.user_guides = std::mem::take(&mut record.user_guides);
    player.region = std::mem::take(&mut record.region);
    player.favors = std::mem::take(&mut record.favors)
        .into_iter()
        .map(|favor| DcNetDataFavor {
            id: favor.character_id,
            exp: favor.exp,
            lv: favor.level,
        })
        .collect();
    player.favor_day = std::mem::take(&mut record.favor_day);
    player.favor_touches = std::mem::take(&mut record.favor_touches);
    player.sms = std::mem::take(&mut record.sms)
        .into_iter()
        .map(|sms| DcNetDataSms {
            id: sms.group_id,
            read: sms.read,
            selected: sms.selected,
        })
        .collect();
}
