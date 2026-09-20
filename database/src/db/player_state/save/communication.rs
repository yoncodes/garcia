use sqlx::SqliteConnection;

use crate::models::game::player_state::PlayerRecord;

pub(super) async fn save(tx: &mut SqliteConnection, player: &PlayerRecord) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM player_checked_red_dots WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for id in &player.checked_red_dots {
        sqlx::query("INSERT INTO player_checked_red_dots (uid, id) VALUES (?, ?)")
            .bind(player.uid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM player_archive_unlocks WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for archive in &player.archive_unlocks {
        sqlx::query(
            "INSERT INTO player_archive_unlocks (uid, archive_id, unlocked_at) VALUES (?, ?, ?)",
        )
        .bind(player.uid)
        .bind(archive.archive_id)
        .bind(archive.unlocked_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM player_mails WHERE uid = ?")
        .bind(player.uid)
        .execute(&mut *tx)
        .await?;
    for mail in &player.mails {
        sqlx::query(
            "INSERT INTO player_mails (
                uid, email_id, is_read, taken, sent_at, sender, title, content, expires_at,
                sys_mail_id, parameter
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(player.uid)
        .bind(mail.email_id)
        .bind(mail.is_read)
        .bind(mail.taken)
        .bind(mail.sent_at)
        .bind(mail.sender)
        .bind(&mail.title)
        .bind(&mail.content)
        .bind(mail.expires_at)
        .bind(mail.sys_mail_id)
        .bind(&mail.parameter)
        .execute(&mut *tx)
        .await?;
        for (position, gift) in mail.gifts.iter().enumerate() {
            sqlx::query(
                "INSERT INTO player_mail_gifts (
                    uid, email_id, position, reward, reward_type, amount
                 ) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(player.uid)
            .bind(mail.email_id)
            .bind(position as i32)
            .bind(gift.reward)
            .bind(gift.reward_type)
            .bind(gift.amount)
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(())
}
