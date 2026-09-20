use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_tp_map WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for entry in &player.tp_map {
        sqlx::query("INSERT INTO player_tp_map (uid, key, value) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(&entry.key)
            .bind(&entry.value)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_role_attrs WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for attr in &player.role_attrs {
        sqlx::query(
            "INSERT INTO player_role_attrs (uid, role_id, mp, ep, hp) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(attr.role_id)
        .bind(attr.mp)
        .bind(attr.ep)
        .bind(attr.hp)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_playing_port WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    if let Some(port) = &player.playing_port {
        sqlx::query(
            "INSERT INTO player_playing_port (uid, port_index_id, port_info) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(port.port_index_id)
        .bind(&port.port_info)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_version_task_claims WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for task_id in &player.version_task_claims {
        sqlx::query("INSERT INTO player_version_task_claims (uid, task_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_missions WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for mission in &player.missions {
        sqlx::query(
            "INSERT INTO player_missions (
                uid, misson_id, total_num, curr_num, taken, take_time, type, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(mission.misson_id)
        .bind(mission.total_num)
        .bind(mission.curr_num)
        .bind(mission.taken)
        .bind(mission.take_time)
        .bind(mission.mission_type)
        .bind(mission.created_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_formations WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for formation in &player.formations {
        sqlx::query("INSERT INTO player_formations (uid, formation_id, remark) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(formation.formation_id)
            .bind(&formation.remark)
            .execute(&mut *tx)
            .await?;
        for (position, member) in formation.positions.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_formation_positions (
                    uid, formation_id, position, game_role_id, key_num
                 ) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(formation.formation_id)
            .bind(position as i64)
            .bind(member.game_role_id)
            .bind(member.key_num)
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query("DELETE FROM player_tasks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for task in &player.tasks {
        sqlx::query(
            "INSERT INTO player_tasks (uid, id, status, picked_at, progress, total)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(task.id)
        .bind(task.status)
        .bind(task.picked_at)
        .bind(task.progress)
        .bind(task.total)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_interact_objs WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for object in &player.interact_objs {
        sqlx::query(
            "INSERT INTO player_interact_objs (uid, object_id, interactive, status, count)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(&object.object_id)
        .bind(object.interactive)
        .bind(object.status)
        .bind(object.count)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_challenges WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for challenge in &player.challenges {
        sqlx::query(
            "INSERT INTO player_challenges (uid, id, stars, finished, claimed)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(&challenge.id)
        .bind(challenge.stars)
        .bind(challenge.finished)
        .bind(challenge.claimed)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_activity_challenge_points WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for reward_id in &player.activity_challenge_point_claims {
        sqlx::query("INSERT INTO player_activity_challenge_points (uid, reward_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(reward_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_albums WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for album in &player.albums {
        sqlx::query("INSERT INTO player_albums (uid, id, status, sort) VALUES (?, ?, ?, ?)")
            .bind(player.uid)
            .bind(album.id)
            .bind(album.status)
            .bind(album.sort)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_dense_fogs WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.dense_fogs {
        sqlx::query("INSERT INTO player_dense_fogs (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_ports WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for port in &player.ports {
        sqlx::query(
            "INSERT INTO player_ports (
                uid, id, pass_cnt, is_c, is_f, port_id, s1, s2, s3, t_cnt, updated_at, all_cnt
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(port.id)
        .bind(port.pass_cnt)
        .bind(port.is_c)
        .bind(port.is_f)
        .bind(port.port_id)
        .bind(port.s1)
        .bind(port.s2)
        .bind(port.s3)
        .bind(port.t_cnt)
        .bind(port.updated_at)
        .bind(port.all_cnt)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_dungeons WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.completed_dungeons {
        sqlx::query("INSERT INTO player_dungeons (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_dungeon_clears WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for clear in &player.dungeon_clears {
        sqlx::query(
            "INSERT INTO player_dungeon_clears (uid, dungeon_type, count) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(clear.dungeon_type)
        .bind(clear.count)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}
