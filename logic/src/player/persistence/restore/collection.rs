use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    player.collections = std::mem::take(&mut record.collections)
        .into_iter()
        .map(|collection| DcNetDataCollection {
            cid: collection.cid,
            in_time: collection.in_time,
            reward: collection.reward,
        })
        .collect();
    for reward in std::mem::take(&mut record.collection_suit_rewards) {
        if let Some(state) = player
            .collection_suit_rewards
            .iter_mut()
            .find(|state| state.sid == reward.suit_id)
        {
            state.steps.push(reward.step);
        } else {
            player
                .collection_suit_rewards
                .push(DcNetDataCollectionSuitReward {
                    sid: reward.suit_id,
                    steps: vec![reward.step],
                });
        }
    }
    player.collection_places = std::mem::take(&mut record.collection_places)
        .into_iter()
        .map(|place| DcNetDataCollectionPlaceInfo {
            cid: place.collection_id,
            pid: place.platform_id,
        })
        .collect();
}
