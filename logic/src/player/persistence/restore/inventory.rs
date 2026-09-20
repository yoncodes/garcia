use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    if !record.items.is_empty() {
        player.items = std::mem::take(&mut record.items)
            .into_iter()
            .map(|item| DcNetDataItem {
                user_item_id: item.user_item_id,
                item_id: item.item_id,
                amount: item.amount,
                remain_sec: item.remain_sec,
                quality: item.quality,
            })
            .collect();
    }
    player.item_acquired = std::mem::take(&mut record.item_acquired)
        .into_iter()
        .map(|item| (item.item_id, item.amount))
        .collect();
    player.item_spent = std::mem::take(&mut record.item_spent)
        .into_iter()
        .map(|item| (item.item_id, item.amount))
        .collect();
    player.partners = std::mem::take(&mut record.partners)
        .into_iter()
        .map(|partner| DcNetDataPartner {
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
        })
        .collect();
}
