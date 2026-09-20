use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let checkin_rows = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT aid, check_days, last_check_day
         FROM player_checkins WHERE uid = ? ORDER BY aid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let mut checkins = Vec::with_capacity(checkin_rows.len());
    for (aid, check_days, last_check_day) in checkin_rows {
        let claimed_days = sqlx::query_scalar::<_, i32>(
            "SELECT get_day FROM player_checkin_claims
             WHERE uid = ? AND aid = ? ORDER BY get_day",
        )
        .bind(uid)
        .bind(aid)
        .fetch_all(pool)
        .await?;
        checkins.push(CheckinRecord {
            aid,
            check_days,
            last_check_day,
            claimed_days,
        });
    }

    player.checkins = checkins;
    Ok(())
}
