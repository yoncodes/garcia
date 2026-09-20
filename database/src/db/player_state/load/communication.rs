use sqlx::SqlitePool;

use crate::models::game::player_state::*;

pub(super) async fn load(
    pool: &SqlitePool,
    uid: i64,
    player: &mut PlayerRecord,
) -> sqlx::Result<()> {
    let city_guides =
        sqlx::query_scalar("SELECT id FROM player_city_guides WHERE uid = ? ORDER BY rowid")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let user_guides =
        sqlx::query_scalar("SELECT gid FROM player_user_guides WHERE uid = ? ORDER BY rowid")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let favors = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT character_id, level, exp FROM player_favors WHERE uid = ? ORDER BY character_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(character_id, level, exp)| FavorRecord {
        character_id,
        level,
        exp,
    })
    .collect();
    let (favor_day, favor_touches) = sqlx::query_as::<_, (i32, i32)>(
        "SELECT day, touches FROM player_favor_daily WHERE uid = ?",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .unwrap_or((i32::MIN, 0));
    let sms_rows = sqlx::query_as::<_, (i32, bool, String)>(
        "SELECT group_id, read, selected FROM player_sms WHERE uid = ? ORDER BY group_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let sms = sms_rows
        .into_iter()
        .map(|(group_id, read, selected)| SmsRecord {
            group_id,
            read,
            selected: selected
                .split(',')
                .filter_map(|value| value.parse().ok())
                .collect(),
        })
        .collect();
    let checked_red_dots =
        sqlx::query_scalar("SELECT id FROM player_checked_red_dots WHERE uid = ? ORDER BY id")
            .bind(uid)
            .fetch_all(pool)
            .await?;
    let mail_rows =
        sqlx::query_as::<_, (i64, i32, i32, i32, i32, String, String, i32, i32, String)>(
            "SELECT email_id, is_read, taken, sent_at, sender, title, content, expires_at,
                sys_mail_id, parameter
         FROM player_mails WHERE uid = ? ORDER BY sent_at DESC, email_id DESC",
        )
        .bind(uid)
        .fetch_all(pool)
        .await?;
    let mut mails = Vec::with_capacity(mail_rows.len());
    for (
        email_id,
        is_read,
        taken,
        sent_at,
        sender,
        title,
        content,
        expires_at,
        sys_mail_id,
        parameter,
    ) in mail_rows
    {
        let gifts = sqlx::query_as::<_, (i32, i32, i32)>(
            "SELECT reward, reward_type, amount FROM player_mail_gifts
             WHERE uid = ? AND email_id = ? ORDER BY position",
        )
        .bind(uid)
        .bind(email_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(reward, reward_type, amount)| MailGiftRecord {
            reward,
            reward_type,
            amount,
        })
        .collect();
        mails.push(MailRecord {
            email_id,
            is_read,
            taken,
            sent_at,
            sender,
            title,
            content,
            gifts,
            expires_at,
            sys_mail_id,
            parameter,
        });
    }
    let locals = sqlx::query_as::<_, (i32, String, String)>(
        "SELECT region, local, local2 FROM player_locals WHERE uid = ? ORDER BY region",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(region, local, local2)| LocalRecord {
        region,
        local,
        local2,
    })
    .collect();

    player.city_guides = city_guides;
    player.user_guides = user_guides;
    player.favors = favors;
    player.favor_day = favor_day;
    player.favor_touches = favor_touches;
    player.sms = sms;
    player.checked_red_dots = checked_red_dots;
    player.mails = mails;
    player.locals = locals;
    Ok(())
}
