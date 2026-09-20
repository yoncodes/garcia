use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_items WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for item in &player.items {
        sqlx::query(
            "INSERT INTO player_items (
                uid, user_item_id, item_id, amount, remain_sec, quality
             ) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(item.user_item_id)
        .bind(item.item_id)
        .bind(item.amount)
        .bind(item.remain_sec)
        .bind(item.quality)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_item_acquired WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for item in &player.item_acquired {
        sqlx::query("INSERT INTO player_item_acquired (uid, item_id, amount) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(item.item_id)
            .bind(item.amount)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_item_spent WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for item in &player.item_spent {
        sqlx::query("INSERT INTO player_item_spent (uid, item_id, amount) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(item.item_id)
            .bind(item.amount)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_partners WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for partner in &player.partners {
        sqlx::query(
            "INSERT INTO player_partners (
                uid, brek, exp, group_id, id, locked, lv, partner_id, quality, reson_lv,
                skill_lv, creat_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(partner.brek)
        .bind(partner.exp)
        .bind(partner.group_id)
        .bind(partner.id)
        .bind(partner.locked)
        .bind(partner.lv)
        .bind(partner.partner_id)
        .bind(partner.quality)
        .bind(partner.reson_lv)
        .bind(partner.skill_lv)
        .bind(partner.creat_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_equips WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for equip in &player.equips {
        sqlx::query(
            "INSERT INTO player_equips (
                uid, equip_id, exp, user_equip_id, in_group, locked, level, equiped_role, pos,
                main_words_id, quality, minnum, tmp_word, tmp_word_idx, creat_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(equip.equip_id)
        .bind(equip.exp)
        .bind(equip.user_equip_id)
        .bind(equip.in_group)
        .bind(equip.locked)
        .bind(equip.level)
        .bind(equip.equiped_role)
        .bind(equip.pos)
        .bind(equip.main_words_id)
        .bind(equip.quality)
        .bind(equip.minnum)
        .bind(equip.tmp_word)
        .bind(equip.tmp_word_idx)
        .bind(equip.creat_at)
        .execute(&mut *tx)
        .await?;
        for (position, word_id) in equip.deputy_words_id.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_equip_deputy_words
                 (uid, user_equip_id, position, word_id) VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(equip.user_equip_id)
            .bind(position as i32)
            .bind(word_id)
            .execute(&mut *tx)
            .await?;
        }
        for (position, word_id) in equip.random_words_id.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_equip_random_words
                 (uid, user_equip_id, position, word_id) VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(equip.user_equip_id)
            .bind(position as i32)
            .bind(word_id)
            .execute(&mut *tx)
            .await?;
        }
        for (position, word_id) in equip.planned_words_id.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_equip_planned_words
                 (uid, user_equip_id, position, word_id) VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(equip.user_equip_id)
            .bind(position as i32)
            .bind(word_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query("DELETE FROM player_equipment_group_items WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM player_equipment_groups WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for group in &player.equipment_groups {
        sqlx::query(
            "INSERT INTO player_equipment_groups (uid, group_id, game_role_id, group_name)
             VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(group.id)
        .bind(group.game_role_id)
        .bind(&group.group_name)
        .execute(&mut *tx)
        .await?;
        for (position, equip_id) in group.user_equip_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_equipment_group_items
                 (uid, group_id, position, user_equip_id) VALUES (?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(group.id)
            .bind(position as i32)
            .bind(equip_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(())
}
