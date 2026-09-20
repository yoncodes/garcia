use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.version_task_claims = player.version_task_claims.clone();
    record.missions = player
        .missions
        .iter()
        .map(
            |mission| database::models::game::player_state::MissionRecord {
                misson_id: mission.misson_id,
                total_num: mission.total_num,
                curr_num: mission.curr_num,
                taken: mission.taken,
                take_time: mission.take_time,
                mission_type: mission.r#type,
                created_at: mission.created_at,
            },
        )
        .collect();
    record.formations = player
        .formations
        .iter()
        .map(
            |formation| database::models::game::player_state::FormationRecord {
                formation_id: formation.formation_id,
                remark: formation.remark.clone(),
                positions: formation
                    .poss
                    .iter()
                    .map(
                        |position| database::models::game::player_state::FormationPositionRecord {
                            game_role_id: position.game_role_id,
                            key_num: position.key_num,
                        },
                    )
                    .collect(),
            },
        )
        .collect();
    record.tasks = player
        .tasks
        .iter()
        .map(|task| database::models::game::player_state::TaskRecord {
            id: task.id,
            status: task.status,
            picked_at: task.picked_at,
            progress: task.progress,
            total: task.total,
        })
        .collect();
    record.role_progress = player
        .roles
        .iter()
        .filter_map(|role| {
            let info = role.role_basic_info.as_ref()?;
            Some(database::models::game::player_state::RoleProgressRecord {
                role_id: info.game_role_id,
                level: info.level,
                exp: info.exp,
                user_partner_id: info.user_partner_id,
                position: info.position,
                maid_qua: info.maid_qua,
                skin_id: info.skin_id,
                appear_skill_key: info.appear_skill_key,
                element: info.element,
                element4call: info.element4call,
                cur_mc: info.cur_mc,
                awards: info.awards.clone(),
                talents: role
                    .talents
                    .iter()
                    .map(|talent| (talent.position, talent.lv))
                    .collect(),
            })
        })
        .collect();
    record.daily_day = player.daily_day;
    record.daily_activity = player.daily_activity;
    record.daily_reward_progress = player.daily_reward_progress;
    record.heat_exchange_day = player.heat_exchange_day;
    record.heat_exchange_count = player.heat_exchange_count;
    record.daily_tasks = player
        .daily_tasks
        .iter()
        .map(
            |task| database::models::game::player_state::DailyTaskRecord {
                id: task.id,
                taken: task.taken,
                progress: task.progress,
                total: task.total,
            },
        )
        .collect();
}
