use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_skins WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for skin_id in &player.skins {
        sqlx::query("INSERT INTO player_skins (uid, skin_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(skin_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_ship_tags WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for tag_id in &player.ship_tags {
        sqlx::query("INSERT INTO player_ship_tags (uid, tag_id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_sms WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for sms in &player.sms {
        let selected = sms
            .selected
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        sqlx::query("INSERT INTO player_sms (uid, group_id, read, selected) VALUES (?, ?, ?, ?)")
            .bind(player.uid)
            .bind(sms.group_id)
            .bind(sms.read)
            .bind(selected)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_favors WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for favor in &player.favors {
        sqlx::query(
            "INSERT INTO player_favors (uid, character_id, level, exp) VALUES (?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(favor.character_id)
        .bind(favor.level)
        .bind(favor.exp)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO player_favor_daily (uid, day, touches) VALUES (?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET day = excluded.day, touches = excluded.touches",
    )
    .bind(player.uid)
    .bind(player.favor_day)
    .bind(player.favor_touches)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM player_city_guides WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.city_guides {
        sqlx::query("INSERT INTO player_city_guides (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_user_guides WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for gid in &player.user_guides {
        sqlx::query("INSERT INTO player_user_guides (uid, gid) VALUES (?, ?)")
            .bind(player.uid)
            .bind(gid)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM player_locals WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for local in &player.locals {
        sqlx::query("INSERT INTO player_locals (uid, region, local, local2) VALUES (?, ?, ?, ?)")
            .bind(player.uid)
            .bind(local.region)
            .bind(&local.local)
            .bind(&local.local2)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}
