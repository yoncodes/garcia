use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.items = player
        .items
        .iter()
        .map(|item| database::models::game::player_state::ItemRecord {
            user_item_id: item.user_item_id,
            item_id: item.item_id,
            amount: item.amount,
            remain_sec: item.remain_sec,
            quality: item.quality,
        })
        .collect();
    record.item_acquired = player
        .item_acquired
        .iter()
        .map(
            |(&item_id, &amount)| database::models::game::player_state::ItemAcquiredRecord {
                item_id,
                amount,
            },
        )
        .collect();
    record.item_spent = player
        .item_spent
        .iter()
        .map(
            |(&item_id, &amount)| database::models::game::player_state::ItemSpentRecord {
                item_id,
                amount,
            },
        )
        .collect();
    record.partners = player
        .partners
        .iter()
        .map(
            |partner| database::models::game::player_state::PartnerRecord {
                brek: partner.brek,
                exp: partner.exp,
                group_id: partner.group_id,
                id: partner.id,
                locked: partner.locked,
                lv: partner.lv,
                partner_id: partner.partner_id,
                quality: partner.quality,
                reson_lv: partner.reson_lv,
                skill_lv: partner.skill_lv,
                creat_at: partner.creat_at,
            },
        )
        .collect();
}
