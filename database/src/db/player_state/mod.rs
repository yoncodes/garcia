mod load;
mod save;

pub use load::load;
pub use save::save;

pub async fn update_last_login(
    pool: &sqlx::SqlitePool,
    uid: i64,
    last_login_time: i32,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE players SET last_login_time = ? WHERE uid = ?")
        .bind(last_login_time)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn exists(pool: &sqlx::SqlitePool, uid: i64) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM players WHERE uid = ?)")
        .bind(uid)
        .fetch_one(pool)
        .await
}

#[cfg(test)]
mod tests;
