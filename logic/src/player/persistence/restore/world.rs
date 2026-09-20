use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
    tables: &GameTables,
) {
    player.tp_map = std::mem::take(&mut record.tp_map)
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect();
    player.role_attrs = std::mem::take(&mut record.role_attrs)
        .into_iter()
        .map(|attr| DcNetDataRoleAttrInfo {
            role_id: attr.role_id,
            mp: attr.mp,
            ep: attr.ep,
            hp: attr.hp,
        })
        .collect();
    player.playing_port = std::mem::take(&mut record.playing_port).map(|port| PlayingPortState {
        port_index_id: port.port_index_id,
        port_info: port.port_info,
    });
    player.version_task_claims = std::mem::take(&mut record.version_task_claims);
    player.locals = std::mem::take(&mut record.locals)
        .into_iter()
        .map(|local| DcNetDataLocal {
            region: local.region,
            local: local.local,
            local2: local.local2,
        })
        .collect();
    player.missions = std::mem::take(&mut record.missions)
        .into_iter()
        .map(|mission| DcNetDataMissonInfo {
            misson_id: mission.misson_id,
            total_num: mission.total_num,
            curr_num: mission.curr_num,
            taken: mission.taken,
            take_time: mission.take_time,
            r#type: mission.mission_type,
            created_at: mission.created_at,
        })
        .collect();
    player.formations = std::mem::take(&mut record.formations)
        .into_iter()
        .map(|formation| DcNetDataFormation {
            formation_id: formation.formation_id,
            remark: formation.remark,
            poss: formation
                .positions
                .into_iter()
                .map(|position| DcNetDataFormationPos {
                    game_role_id: position.game_role_id,
                    key_num: position.key_num,
                })
                .collect(),
        })
        .collect();
    for formation_id in 0..=tables.cultivation_constants.fixed_formation_count {
        if !player
            .formations
            .iter()
            .any(|formation| formation.formation_id == formation_id)
        {
            player.formations.push(DcNetDataFormation {
                formation_id,
                ..Default::default()
            });
        }
    }
    player.tasks = std::mem::take(&mut record.tasks)
        .into_iter()
        .map(|task| DcNetDataTaskStatus {
            id: task.id,
            status: task.status,
            picked_at: task.picked_at,
            progress: task.progress,
            total: task.total,
        })
        .collect();
    if !record.interact_objs.is_empty() {
        player.interact_objs = std::mem::take(&mut record.interact_objs)
            .into_iter()
            .map(|object| DcNetDataInteractObj {
                object_id: object.object_id,
                interactive: object.interactive,
                status: object.status,
                count: object.count,
            })
            .collect();
    }
    if !record.challenges.is_empty() {
        player.challenges = std::mem::take(&mut record.challenges)
            .into_iter()
            .map(|challenge| DcNetDataChallenge {
                id: challenge.id,
                stars: challenge.stars,
                finished: challenge.finished,
                claimed: challenge.claimed,
            })
            .collect();
    }
    if !record.albums.is_empty() {
        player.albums = std::mem::take(&mut record.albums)
            .into_iter()
            .map(|album| DcNetDataAlbums {
                id: album.id,
                status: album.status,
                sort: album.sort,
            })
            .collect();
    }
    if !record.dense_fogs.is_empty() {
        player.dense_fogs = std::mem::take(&mut record.dense_fogs);
    }
    player.ports = std::mem::take(&mut record.ports)
        .into_iter()
        .map(|port| DcNetDataPort {
            pass_cnt: port.pass_cnt,
            is_c: port.is_c,
            is_f: port.is_f,
            port_id: port.port_id,
            s1: port.s1,
            s2: port.s2,
            s3: port.s3,
            t_cnt: port.t_cnt,
            updated_at: port.updated_at,
            id: port.id,
            all_cnt: port.all_cnt,
        })
        .collect();
    player.completed_dungeons = std::mem::take(&mut record.completed_dungeons);
    player.dungeon_clears = std::mem::take(&mut record.dungeon_clears)
        .into_iter()
        .map(|clear| (clear.dungeon_type, clear.count))
        .collect();
}
