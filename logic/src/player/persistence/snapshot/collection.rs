use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.collections = player
        .collections
        .iter()
        .map(
            |collection| database::models::game::player_state::CollectionRecord {
                cid: collection.cid,
                in_time: collection.in_time,
                reward: collection.reward,
            },
        )
        .collect();
    record.collection_suit_rewards = player
        .collection_suit_rewards
        .iter()
        .flat_map(|reward| {
            reward.steps.iter().map(|&step| {
                database::models::game::player_state::CollectionSuitRewardRecord {
                    suit_id: reward.sid,
                    step,
                }
            })
        })
        .collect();
    record.collection_places = player
        .collection_places
        .iter()
        .map(
            |place| database::models::game::player_state::CollectionPlaceRecord {
                collection_id: place.cid,
                platform_id: place.pid,
            },
        )
        .collect();
}
