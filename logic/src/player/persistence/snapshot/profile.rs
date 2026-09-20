use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.profile_unlocks = player
        .profile_avatars
        .iter()
        .map(
            |profile| database::models::game::player_state::ProfileUnlockRecord {
                profile_type: 1,
                profile_id: profile.id,
            },
        )
        .chain(player.profile_frames.iter().map(|profile| {
            database::models::game::player_state::ProfileUnlockRecord {
                profile_type: 2,
                profile_id: profile.id,
            }
        }))
        .chain(player.profile_titles.iter().map(|profile| {
            database::models::game::player_state::ProfileUnlockRecord {
                profile_type: 3,
                profile_id: profile.id,
            }
        }))
        .chain(player.profile_cards.iter().map(|profile| {
            database::models::game::player_state::ProfileUnlockRecord {
                profile_type: 4,
                profile_id: profile.id,
            }
        }))
        .collect();
    record.skins = player.skins.clone();
    record.ship_tags = player.ship_tags.clone();
    record.archive_unlocks = player
        .archive_unlocks
        .iter()
        .map(|(&archive_id, &unlocked_at)| {
            database::models::game::player_state::ArchiveUnlockRecord {
                archive_id,
                unlocked_at,
            }
        })
        .collect();
    record.city_guides = player.city_guides.clone();
    record.user_guides = player.user_guides.clone();
    record.favors = player
        .favors
        .iter()
        .map(|favor| database::models::game::player_state::FavorRecord {
            character_id: favor.id,
            level: favor.lv,
            exp: favor.exp,
        })
        .collect();
    record.favor_day = player.favor_day;
    record.favor_touches = player.favor_touches;
    record.sms = player
        .sms
        .iter()
        .map(|sms| database::models::game::player_state::SmsRecord {
            group_id: sms.id,
            read: sms.read,
            selected: sms.selected.clone(),
        })
        .collect();
    record.locals = player
        .locals
        .iter()
        .map(|local| database::models::game::player_state::LocalRecord {
            region: local.region,
            local: local.local.clone(),
            local2: local.local2.clone(),
        })
        .collect();
}
