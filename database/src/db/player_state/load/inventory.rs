use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let items = sqlx::query_as::<_, (i64, i32, i32, i32, i32)>(
        "SELECT user_item_id, item_id, amount, remain_sec, quality
         FROM player_items WHERE uid = ? ORDER BY item_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(user_item_id, item_id, amount, remain_sec, quality)| ItemRecord {
            user_item_id,
            item_id,
            amount,
            remain_sec,
            quality,
        },
    )
    .collect();
    let team_cores = sqlx::query_as::<_, (i32, i64, bool, i64, i32)>(
        "SELECT pos, instance_id, locked, equipped_id, core_id
         FROM player_team_cores WHERE uid = ? ORDER BY instance_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(pos, id, locked, equipped_id, core_id)| TeamCoreRecord {
        pos,
        id,
        locked,
        equipped_id,
        core_id,
    })
    .collect();
    let team_equips = sqlx::query_as::<_, (i32, i64, bool, i32, i32, i32, i32, i32)>(
        "SELECT equipped_formation_id, instance_id, locked, team_equip_id, level, exp,
                main_words_id, pos_num
         FROM player_team_equips WHERE uid = ? ORDER BY instance_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(equipped_formation_id, id, locked, team_equip_id, level, exp, main_words_id, pos_num)| {
            TeamEquipRecord {
                equipped_formation_id,
                id,
                locked,
                team_equip_id,
                level,
                exp,
                main_words_id,
                pos_num,
            }
        },
    )
    .collect();
    let skillstones = sqlx::query_as::<_, (i32, i64, bool, i32, i32, i32)>(
        "SELECT position, user_stone_id, locked, equipped_role, stone_id, quality
         FROM player_skillstones WHERE uid = ? ORDER BY user_stone_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(pos, user_stone_id, locked, equipped_role, stone_id, quality)| SkillStoneRecord {
            pos,
            user_stone_id,
            locked,
            equipped_role,
            stone_id,
            quality,
        },
    )
    .collect();
    let feats = sqlx::query_as::<_, (i32, i32)>(
        "SELECT feat_id, status FROM player_feats WHERE uid = ? ORDER BY feat_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, status)| FeatRecord { id, status })
    .collect();
    let item_acquired = sqlx::query_as::<_, (i32, i32)>(
        "SELECT item_id, amount FROM player_item_acquired WHERE uid = ? ORDER BY item_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(item_id, amount)| ItemAcquiredRecord { item_id, amount })
    .collect();
    let item_spent = sqlx::query_as::<_, (i32, i32)>(
        "SELECT item_id, amount FROM player_item_spent WHERE uid = ? ORDER BY item_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(item_id, amount)| ItemSpentRecord { item_id, amount })
    .collect();

    player.items = items;
    player.team_cores = team_cores;
    player.team_equips = team_equips;
    player.skillstones = skillstones;
    player.feats = feats;
    player.item_acquired = item_acquired;
    player.item_spent = item_spent;
    Ok(())
}
