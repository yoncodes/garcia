use sqlx::SqlitePool;

use crate::models::game::player_state::*;
use crate::models::game::world::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let tp_map = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM player_tp_map WHERE uid = ? ORDER BY key",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(key, value)| TpMapRecord { key, value })
    .collect();
    let role_attrs = sqlx::query_as::<_, (i32, i32, i32, i32)>(
        "SELECT role_id, mp, ep, hp FROM player_role_attrs WHERE uid = ? ORDER BY role_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(role_id, mp, ep, hp)| RoleAttrRecord {
        role_id,
        mp,
        ep,
        hp,
    })
    .collect();
    let playing_port = sqlx::query_as::<_, (i32, String)>(
        "SELECT port_index_id, port_info FROM player_playing_port WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .map(|(port_index_id, port_info)| PlayingPortRecord {
        port_index_id,
        port_info,
    });
    let version_task_claims = sqlx::query_scalar(
        "SELECT task_id FROM player_version_task_claims WHERE uid = ? ORDER BY task_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let missions = sqlx::query_as::<_, (i32, i32, i32, bool, i32, i32, i32)>(
        "SELECT misson_id, total_num, curr_num, taken, take_time, type, created_at
         FROM player_missions WHERE uid = ? ORDER BY misson_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(misson_id, total_num, curr_num, taken, take_time, mission_type, created_at)| {
            MissionRecord {
                misson_id,
                total_num,
                curr_num,
                taken,
                take_time,
                mission_type,
                created_at,
            }
        },
    )
    .collect();
    let formation_rows = sqlx::query_as::<_, (i32, String)>(
        "SELECT formation_id, remark FROM player_formations
         WHERE uid = ? ORDER BY formation_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let mut formations = Vec::with_capacity(formation_rows.len());
    for (formation_id, remark) in formation_rows {
        let positions = sqlx::query_as::<_, (i32, i32)>(
            "SELECT game_role_id, key_num FROM player_formation_positions
             WHERE uid = ? AND formation_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(formation_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(game_role_id, key_num)| FormationPositionRecord {
            game_role_id,
            key_num,
        })
        .collect();
        formations.push(FormationRecord {
            formation_id,
            remark,
            positions,
        });
    }
    let tasks = sqlx::query_as::<_, (i32, i32, i64, i32, i32)>(
        "SELECT id, status, picked_at, progress, total
         FROM player_tasks WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, status, picked_at, progress, total)| TaskRecord {
        id,
        status,
        picked_at,
        progress,
        total,
    })
    .collect();
    let interact_objs = sqlx::query_as::<_, (String, bool, i32, i32)>(
        "SELECT object_id, interactive, status, count
         FROM player_interact_objs WHERE uid = ? ORDER BY rowid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(object_id, interactive, status, count)| InteractRecord {
        object_id,
        interactive,
        status,
        count,
    })
    .collect();
    let challenges = sqlx::query_as::<_, (String, i32, bool, bool)>(
        "SELECT id, stars, finished, claimed
         FROM player_challenges WHERE uid = ? ORDER BY rowid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, stars, finished, claimed)| ChallengeRecord {
        id,
        stars,
        finished,
        claimed,
    })
    .collect();
    let activity_challenge_point_claims = sqlx::query_scalar(
        "SELECT reward_id FROM player_activity_challenge_points WHERE uid = ? ORDER BY reward_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let albums = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT id, status, sort FROM player_albums WHERE uid = ? ORDER BY rowid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, status, sort)| AlbumRecord { id, status, sort })
    .collect();
    let dense_fogs =
        sqlx::query_scalar("SELECT id FROM player_dense_fogs WHERE uid = ? ORDER BY rowid")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let ports = sqlx::query_as::<_, (i32, i32, bool, bool, i32, i32, i32, i32, i32, i32, i32)>(
        "SELECT id, pass_cnt, is_c, is_f, port_id, s1, s2, s3, t_cnt, updated_at, all_cnt
         FROM player_ports WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(id, pass_cnt, is_c, is_f, port_id, s1, s2, s3, t_cnt, updated_at, all_cnt)| PortRecord {
            id,
            pass_cnt,
            is_c,
            is_f,
            port_id,
            s1,
            s2,
            s3,
            t_cnt,
            updated_at,
            all_cnt,
        },
    )
    .collect();
    let completed_dungeons =
        sqlx::query_scalar("SELECT id FROM player_dungeons WHERE uid = ? ORDER BY id")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let dungeon_clears = sqlx::query_as::<_, (i32, i32)>(
        "SELECT dungeon_type, count FROM player_dungeon_clears WHERE uid = ? ORDER BY dungeon_type",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(dungeon_type, count)| DungeonClearRecord {
        dungeon_type,
        count,
    })
    .collect();

    player.tp_map = tp_map;
    player.role_attrs = role_attrs;
    player.playing_port = playing_port;
    player.version_task_claims = version_task_claims;
    player.missions = missions;
    player.formations = formations;
    player.tasks = tasks;
    player.interact_objs = interact_objs;
    player.challenges = challenges;
    player.activity_challenge_point_claims = activity_challenge_point_claims;
    player.albums = albums;
    player.dense_fogs = dense_fogs;
    player.ports = ports;
    player.completed_dungeons = completed_dungeons;
    player.dungeon_clears = dungeon_clears;
    Ok(())
}
