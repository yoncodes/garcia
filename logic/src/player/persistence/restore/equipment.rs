use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
    tables: &GameTables,
) {
    player.equipment_word_plans = record
        .equips
        .iter()
        .filter(|equip| !equip.planned_words_id.is_empty())
        .map(|equip| (equip.user_equip_id, equip.planned_words_id.clone()))
        .collect();
    player.equips = std::mem::take(&mut record.equips)
        .into_iter()
        .map(|equip| DcNetDataEquip {
            equip_id: equip.equip_id,
            exp: equip.exp,
            user_equip_id: equip.user_equip_id,
            in_group: equip.in_group,
            locked: equip.locked,
            level: equip.level,
            equiped_role: equip.equiped_role,
            pos: equip.pos,
            main_words_id: equip.main_words_id,
            deputy_words_id: equip.deputy_words_id,
            quality: equip.quality,
            random_words_id: equip.random_words_id,
            minnum: equip.minnum,
            tmp_word: equip.tmp_word,
            tmp_word_idx: equip.tmp_word_idx,
            creat_at: equip.creat_at,
        })
        .collect();
    for equip in &mut player.equips {
        let mut planned_words = player
            .equipment_word_plans
            .remove(&equip.user_equip_id)
            .unwrap_or_default();
        crate::player::equipment::normalize_equipment_words(equip, &mut planned_words, tables);
        if !planned_words.is_empty() {
            player
                .equipment_word_plans
                .insert(equip.user_equip_id, planned_words);
        }
    }
    player.equipment_groups = std::mem::take(&mut record.equipment_groups)
        .into_iter()
        .map(|group| DcNetDataEquipsGroup {
            id: group.id,
            game_role_id: group.game_role_id,
            group_name: group.group_name,
            user_equip_ids: group.user_equip_ids,
        })
        .collect();
    player.team_cores = std::mem::take(&mut record.team_cores)
        .into_iter()
        .map(|core| DcNetDataTeamCore {
            pos: core.pos,
            id: core.id,
            locked: core.locked,
            equiped_id: core.equipped_id,
            core_id: core.core_id,
        })
        .collect();
    player.team_equips = std::mem::take(&mut record.team_equips)
        .into_iter()
        .map(|equip| DcNetDataTeamEquip {
            equiped_fid: equip.equipped_formation_id,
            id: equip.id,
            locked: equip.locked,
            te_id: equip.team_equip_id,
            lv: equip.level,
            exp: equip.exp,
            main_words_id: equip.main_words_id,
            pos_num: equip.pos_num,
        })
        .collect();
    player.skillstones = std::mem::take(&mut record.skillstones)
        .into_iter()
        .map(|stone| DcNetDataSkillStone {
            pos: stone.pos,
            user_stone_id: stone.user_stone_id,
            locked: stone.locked,
            equiped_role: stone.equipped_role,
            stone_id: stone.stone_id,
            quality: stone.quality,
        })
        .collect();
    player.feats = std::mem::take(&mut record.feats)
        .into_iter()
        .map(|feat| DcNetDataFeat {
            id: feat.id,
            status: feat.status,
        })
        .collect();
}
