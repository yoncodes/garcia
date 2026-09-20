use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_team_cores WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for core in &player.team_cores {
        sqlx::query(
            "INSERT INTO player_team_cores
             (uid, instance_id, pos, locked, equipped_id, core_id)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(core.id)
        .bind(core.pos)
        .bind(core.locked)
        .bind(core.equipped_id)
        .bind(core.core_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_team_equips WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for equip in &player.team_equips {
        sqlx::query(
            "INSERT INTO player_team_equips
             (uid, instance_id, equipped_formation_id, locked, team_equip_id, level, exp,
              main_words_id, pos_num)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(equip.id)
        .bind(equip.equipped_formation_id)
        .bind(equip.locked)
        .bind(equip.team_equip_id)
        .bind(equip.level)
        .bind(equip.exp)
        .bind(equip.main_words_id)
        .bind(equip.pos_num)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_skillstones WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for stone in &player.skillstones {
        sqlx::query(
            "INSERT INTO player_skillstones
             (uid, user_stone_id, stone_id, position, locked, equipped_role, quality)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(stone.user_stone_id)
        .bind(stone.stone_id)
        .bind(stone.pos)
        .bind(stone.locked)
        .bind(stone.equipped_role)
        .bind(stone.quality)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_feats WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for feat in &player.feats {
        sqlx::query("INSERT INTO player_feats (uid, feat_id, status) VALUES (?, ?, ?)")
            .bind(player.uid)
            .bind(feat.id)
            .bind(feat.status)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}
