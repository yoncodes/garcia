use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.equips = player
        .equips
        .iter()
        .map(|equip| database::models::game::player_state::EquipRecord {
            equip_id: equip.equip_id,
            exp: equip.exp,
            user_equip_id: equip.user_equip_id,
            in_group: equip.in_group,
            locked: equip.locked,
            level: equip.level,
            equiped_role: equip.equiped_role,
            pos: equip.pos,
            main_words_id: equip.main_words_id,
            deputy_words_id: equip.deputy_words_id.clone(),
            quality: equip.quality,
            random_words_id: equip.random_words_id.clone(),
            planned_words_id: player
                .equipment_word_plans
                .get(&equip.user_equip_id)
                .cloned()
                .unwrap_or_default(),
            minnum: equip.minnum,
            tmp_word: equip.tmp_word,
            tmp_word_idx: equip.tmp_word_idx,
            creat_at: equip.creat_at,
        })
        .collect();
    record.equipment_groups = player
        .equipment_groups
        .iter()
        .map(
            |group| database::models::game::player_state::EquipmentGroupRecord {
                id: group.id,
                game_role_id: group.game_role_id,
                group_name: group.group_name.clone(),
                user_equip_ids: group.user_equip_ids.clone(),
            },
        )
        .collect();
    record.team_cores = player
        .team_cores
        .iter()
        .map(
            |core| database::models::game::player_state::TeamCoreRecord {
                pos: core.pos,
                id: core.id,
                locked: core.locked,
                equipped_id: core.equiped_id,
                core_id: core.core_id,
            },
        )
        .collect();
    record.team_equips = player
        .team_equips
        .iter()
        .map(
            |equip| database::models::game::player_state::TeamEquipRecord {
                equipped_formation_id: equip.equiped_fid,
                id: equip.id,
                locked: equip.locked,
                team_equip_id: equip.te_id,
                level: equip.lv,
                exp: equip.exp,
                main_words_id: equip.main_words_id,
                pos_num: equip.pos_num,
            },
        )
        .collect();
    record.skillstones = player
        .skillstones
        .iter()
        .map(
            |stone| database::models::game::player_state::SkillStoneRecord {
                pos: stone.pos,
                user_stone_id: stone.user_stone_id,
                locked: stone.locked,
                equipped_role: stone.equiped_role,
                stone_id: stone.stone_id,
                quality: stone.quality,
            },
        )
        .collect();
    record.feats = player
        .feats
        .iter()
        .map(|feat| database::models::game::player_state::FeatRecord {
            id: feat.id,
            status: feat.status,
        })
        .collect();
}
