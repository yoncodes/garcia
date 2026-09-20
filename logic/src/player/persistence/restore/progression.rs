use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
    tables: &GameTables,
) {
    for progress in std::mem::take(&mut record.role_progress) {
        if !tables.is_playable_maid(progress.role_id) {
            continue;
        }
        let role_index = player.roles.iter().position(|role| {
            role.role_basic_info
                .as_ref()
                .is_some_and(|info| info.game_role_id == progress.role_id)
        });
        let role_index = role_index.or_else(|| {
            let role = gacha_role(player.uid, progress.role_id, tables)?;
            player.roles.push(role);
            Some(player.roles.len() - 1)
        });
        if let Some(role_index) = role_index {
            let info = player.roles[role_index].role_basic_info.as_mut().unwrap();
            info.level = progress.level;
            info.exp = progress.exp;
            info.user_partner_id = progress.user_partner_id;
            info.position = progress.position;
            info.maid_qua = progress.maid_qua;
            info.skin_id = progress.skin_id;
            info.appear_skill_key = progress.appear_skill_key;
            info.element = progress.element;
            info.element4call = progress.element4call;
            info.cur_mc = progress.cur_mc;
            info.awards = progress.awards;
            for (position, level) in progress.talents {
                if let Some(talent) = player.roles[role_index]
                    .talents
                    .iter_mut()
                    .find(|talent| talent.position == position)
                {
                    talent.lv = level;
                }
            }
        }
    }
    player.sync_role_skillstones();
}
