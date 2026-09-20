use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::models::social::{FriendLink, SocialState};

pub async fn load_state(pool: &SqlitePool, uid: i64, day: i32) -> sqlx::Result<SocialState> {
    let friend_uids = sqlx::query_scalar::<_, i64>(
        "SELECT CASE WHEN uid_low = ? THEN uid_high ELSE uid_low END
         FROM friendships WHERE uid_low = ? OR uid_high = ? ORDER BY created_at, uid_low, uid_high",
    )
    .bind(uid)
    .bind(uid)
    .bind(uid)
    .fetch_all(pool)
    .await?;

    let mut friends = Vec::with_capacity(friend_uids.len());
    for friend_uid in friend_uids {
        let (has_gift, gift_sent) = sqlx::query_as::<_, (bool, bool)>(
            "SELECT
                EXISTS(SELECT 1 FROM friend_gifts
                       WHERE sender_uid = ? AND receiver_uid = ? AND day = ? AND claimed = 0),
                EXISTS(SELECT 1 FROM friend_gifts
                       WHERE sender_uid = ? AND receiver_uid = ? AND day = ?)",
        )
        .bind(friend_uid)
        .bind(uid)
        .bind(day)
        .bind(uid)
        .bind(friend_uid)
        .bind(day)
        .fetch_one(pool)
        .await?;
        friends.push(FriendLink {
            uid: friend_uid,
            has_gift,
            gift_sent,
        });
    }

    let pending_approvals = sqlx::query_scalar(
        "SELECT requester_uid FROM friend_requests WHERE receiver_uid = ? ORDER BY created_at, requester_uid",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    let claimed_gifts = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friend_gifts WHERE receiver_uid = ? AND day = ? AND claimed = 1",
    )
    .bind(uid)
    .bind(day)
    .fetch_one(pool)
    .await?;

    Ok(SocialState {
        friends,
        pending_approvals,
        claimed_gifts,
    })
}

pub async fn send_friend_request(
    pool: &SqlitePool,
    requester_uid: i64,
    receiver_uid: i64,
    now: i32,
) -> sqlx::Result<bool> {
    let (low, high) = ordered_pair(requester_uid, receiver_uid);
    let result = sqlx::query(
        "INSERT INTO friend_requests (requester_uid, receiver_uid, created_at)
         SELECT ?, ?, ?
         WHERE ? <> ?
           AND EXISTS(SELECT 1 FROM players WHERE uid = ?)
           AND NOT EXISTS(SELECT 1 FROM friendships WHERE uid_low = ? AND uid_high = ?)
         ON CONFLICT(requester_uid, receiver_uid) DO NOTHING",
    )
    .bind(requester_uid)
    .bind(receiver_uid)
    .bind(now)
    .bind(requester_uid)
    .bind(receiver_uid)
    .bind(receiver_uid)
    .bind(low)
    .bind(high)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() != 0)
}

pub async fn accept_friend_request(
    pool: &SqlitePool,
    uid: i64,
    requester_uid: i64,
    maximum: i32,
    now: i32,
) -> sqlx::Result<bool> {
    let mut tx = pool.begin().await?;
    let pending = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM friend_requests WHERE requester_uid = ? AND receiver_uid = ?)",
    )
    .bind(requester_uid)
    .bind(uid)
    .fetch_one(&mut *tx)
    .await?;
    if !pending
        || friend_count(&mut tx, uid).await? >= i64::from(maximum)
        || friend_count(&mut tx, requester_uid).await? >= i64::from(maximum)
    {
        return Ok(false);
    }

    let (low, high) = ordered_pair(uid, requester_uid);
    sqlx::query(
        "INSERT OR IGNORE INTO friendships (uid_low, uid_high, created_at) VALUES (?, ?, ?)",
    )
    .bind(low)
    .bind(high)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "DELETE FROM friend_requests
         WHERE (requester_uid = ? AND receiver_uid = ?) OR (requester_uid = ? AND receiver_uid = ?)",
    )
    .bind(requester_uid)
    .bind(uid)
    .bind(uid)
    .bind(requester_uid)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn remove_friend(pool: &SqlitePool, uid: i64, friend_uid: i64) -> sqlx::Result<bool> {
    let (low, high) = ordered_pair(uid, friend_uid);
    let mut tx = pool.begin().await?;
    let result = sqlx::query("DELETE FROM friendships WHERE uid_low = ? AND uid_high = ?")
        .bind(low)
        .bind(high)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM friend_gifts
         WHERE (sender_uid = ? AND receiver_uid = ?) OR (sender_uid = ? AND receiver_uid = ?)",
    )
    .bind(uid)
    .bind(friend_uid)
    .bind(friend_uid)
    .bind(uid)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(result.rows_affected() != 0)
}

pub async fn send_gift(
    pool: &SqlitePool,
    sender_uid: i64,
    receiver_uid: i64,
    day: i32,
) -> sqlx::Result<bool> {
    let (low, high) = ordered_pair(sender_uid, receiver_uid);
    let result = sqlx::query(
        "INSERT INTO friend_gifts (sender_uid, receiver_uid, day)
         SELECT ?, ?, ? WHERE EXISTS(
             SELECT 1 FROM friendships WHERE uid_low = ? AND uid_high = ?
         ) ON CONFLICT(sender_uid, receiver_uid, day) DO NOTHING",
    )
    .bind(sender_uid)
    .bind(receiver_uid)
    .bind(day)
    .bind(low)
    .bind(high)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() != 0)
}

pub async fn claim_gift(
    pool: &SqlitePool,
    receiver_uid: i64,
    sender_uid: i64,
    day: i32,
    maximum: i32,
) -> sqlx::Result<Option<i32>> {
    let mut tx = pool.begin().await?;
    let claimed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friend_gifts WHERE receiver_uid = ? AND day = ? AND claimed = 1",
    )
    .bind(receiver_uid)
    .bind(day)
    .fetch_one(&mut *tx)
    .await?;
    if claimed >= i64::from(maximum) {
        return Ok(None);
    }
    let result = sqlx::query(
        "UPDATE friend_gifts SET claimed = 1
         WHERE sender_uid = ? AND receiver_uid = ? AND day = ? AND claimed = 0",
    )
    .bind(sender_uid)
    .bind(receiver_uid)
    .bind(day)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Ok(None);
    }
    tx.commit().await?;
    Ok(Some((claimed + 1) as i32))
}

async fn friend_count(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM friendships WHERE uid_low = ? OR uid_high = ?")
        .bind(uid)
        .bind(uid)
        .fetch_one(&mut **tx)
        .await
}

fn ordered_pair(left: i64, right: i64) -> (i64, i64) {
    if left < right {
        (left, right)
    } else {
        (right, left)
    }
}

#[cfg(test)]
mod tests;
