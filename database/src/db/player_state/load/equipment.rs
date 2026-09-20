use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let partners = sqlx::query_as::<_, (i32, i32, i32, i64, i32, i32, i32, i32, i32, i32, i64)>(
        "SELECT brek, exp, group_id, id, locked, lv, partner_id, quality, reson_lv, skill_lv,
                creat_at FROM player_partners WHERE uid = ? ORDER BY id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(
            brek,
            exp,
            group_id,
            id,
            locked,
            lv,
            partner_id,
            quality,
            reson_lv,
            skill_lv,
            creat_at,
        )| {
            PartnerRecord {
                brek,
                exp,
                group_id,
                id,
                locked,
                lv,
                partner_id,
                quality,
                reson_lv,
                skill_lv,
                creat_at,
            }
        },
    )
    .collect();
    let mut equips: Vec<EquipRecord> = sqlx::query_as::<
        _,
        (
            i32,
            i32,
            i64,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i64,
        ),
    >(
        "SELECT equip_id, exp, user_equip_id, in_group, locked, level, equiped_role, pos,
                main_words_id, quality, minnum, tmp_word, tmp_word_idx, creat_at
         FROM player_equips WHERE uid = ? ORDER BY user_equip_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(
            equip_id,
            exp,
            user_equip_id,
            in_group,
            locked,
            level,
            equiped_role,
            pos,
            main_words_id,
            quality,
            minnum,
            tmp_word,
            tmp_word_idx,
            creat_at,
        )| EquipRecord {
            equip_id,
            exp,
            user_equip_id,
            in_group,
            locked,
            level,
            equiped_role,
            pos,
            main_words_id,
            deputy_words_id: Vec::new(),
            quality,
            random_words_id: Vec::new(),
            planned_words_id: Vec::new(),
            minnum,
            tmp_word,
            tmp_word_idx,
            creat_at,
        },
    )
    .collect();
    for equip in &mut equips {
        equip.deputy_words_id = sqlx::query_scalar(
            "SELECT word_id FROM player_equip_deputy_words
             WHERE uid = ? AND user_equip_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(equip.user_equip_id)
        .fetch_all(pool)
        .await?;
        equip.random_words_id = sqlx::query_scalar(
            "SELECT word_id FROM player_equip_random_words
             WHERE uid = ? AND user_equip_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(equip.user_equip_id)
        .fetch_all(pool)
        .await?;
        equip.planned_words_id = sqlx::query_scalar(
            "SELECT word_id FROM player_equip_planned_words
             WHERE uid = ? AND user_equip_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(equip.user_equip_id)
        .fetch_all(pool)
        .await?;
    }
    let mut equipment_groups = sqlx::query_as::<_, (i64, i32, String)>(
        "SELECT group_id, game_role_id, group_name
         FROM player_equipment_groups WHERE uid = ? ORDER BY group_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, game_role_id, group_name)| EquipmentGroupRecord {
        id,
        game_role_id,
        group_name,
        user_equip_ids: Vec::new(),
    })
    .collect::<Vec<_>>();
    for group in &mut equipment_groups {
        group.user_equip_ids = sqlx::query_scalar(
            "SELECT user_equip_id FROM player_equipment_group_items
             WHERE uid = ? AND group_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(group.id)
        .fetch_all(pool)
        .await?;
    }

    player.partners = partners;
    player.equips = equips;
    player.equipment_groups = equipment_groups;
    Ok(())
}
