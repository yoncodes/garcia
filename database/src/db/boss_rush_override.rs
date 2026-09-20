use sqlx::SqlitePool;

use crate::models::game::boss_rush::BossRushOverride;

pub async fn current(pool: &SqlitePool) -> sqlx::Result<Option<BossRushOverride>> {
    sqlx::query_as::<_, (i32, i32, i32, bool)>(
        "SELECT event_id, started_at, expires_at, instant_rewards
         FROM boss_rush_override LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(event_id, started_at, expires_at, instant_rewards)| BossRushOverride {
                event_id,
                started_at,
                expires_at,
                instant_rewards,
            },
        )
    })
}

pub async fn active(pool: &SqlitePool, now: i32) -> sqlx::Result<Option<BossRushOverride>> {
    sqlx::query_as::<_, (i32, i32, i32, bool)>(
        "SELECT event_id, started_at, expires_at, instant_rewards FROM boss_rush_override
         WHERE expires_at > ? LIMIT 1",
    )
    .bind(now)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(event_id, started_at, expires_at, instant_rewards)| BossRushOverride {
                event_id,
                started_at,
                expires_at,
                instant_rewards,
            },
        )
    })
}

pub async fn enable(
    pool: &SqlitePool,
    event_id: i32,
    started_at: i32,
    expires_at: i32,
) -> sqlx::Result<()> {
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM boss_rush_override")
        .execute(&mut *transaction)
        .await?;
    sqlx::query(
        "INSERT INTO boss_rush_override
         (event_id, started_at, expires_at, instant_rewards) VALUES (?, ?, ?, 0)",
    )
    .bind(event_id)
    .bind(started_at)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await
}

pub async fn disable(pool: &SqlitePool, event_id: i32, ended_at: i32) -> sqlx::Result<bool> {
    Ok(sqlx::query(
        "UPDATE boss_rush_override
         SET expires_at = MIN(expires_at, ?), instant_rewards = 1
         WHERE event_id = ?",
    )
    .bind(ended_at)
    .bind(event_id)
    .execute(pool)
    .await?
    .rows_affected()
        != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enabling_a_season_replaces_the_previous_override() {
        let pool = crate::connect_memory().await.unwrap();
        enable(&pool, 1001, 100, 200).await.unwrap();
        enable(&pool, 1005, 110, 300).await.unwrap();
        assert_eq!(current(&pool).await.unwrap().unwrap().event_id, 1005);
        assert_eq!(
            active(&pool, 150).await.unwrap(),
            Some(BossRushOverride {
                event_id: 1005,
                started_at: 110,
                expires_at: 300,
                instant_rewards: false,
            })
        );
        assert!(active(&pool, 301).await.unwrap().is_none());
        assert_eq!(current(&pool).await.unwrap().unwrap().event_id, 1005);
        assert!(disable(&pool, 1005, 150).await.unwrap());
        assert_eq!(active(&pool, 150).await.unwrap(), None);
        assert_eq!(current(&pool).await.unwrap().unwrap().expires_at, 150);
        assert!(current(&pool).await.unwrap().unwrap().instant_rewards);
    }
}
