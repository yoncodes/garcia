use sqlx::SqlitePool;

use crate::models::game::gacha_banner::GachaBannerOverride;

pub async fn active(pool: &SqlitePool, now: i32) -> sqlx::Result<Vec<GachaBannerOverride>> {
    sqlx::query_as::<_, (i32, i32)>(
        "SELECT gacha_id, expires_at FROM gacha_banner_overrides
         WHERE expires_at > ? ORDER BY gacha_id",
    )
    .bind(now)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(gacha_id, expires_at)| GachaBannerOverride {
                gacha_id,
                expires_at,
            })
            .collect()
    })
}

pub async fn enable(pool: &SqlitePool, gacha_id: i32, expires_at: i32) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO gacha_banner_overrides (gacha_id, expires_at) VALUES (?, ?)
         ON CONFLICT(gacha_id) DO UPDATE SET expires_at = excluded.expires_at",
    )
    .bind(gacha_id)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn disable(pool: &SqlitePool, gacha_id: i32) -> sqlx::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM gacha_banner_overrides WHERE gacha_id = ?")
            .bind(gacha_id)
            .execute(pool)
            .await?
            .rows_affected()
            != 0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn override_persists_until_expiry_or_deactivation() {
        let pool = crate::connect_memory().await.unwrap();
        enable(&pool, 3, 200).await.unwrap();
        assert_eq!(
            active(&pool, 100).await.unwrap(),
            [GachaBannerOverride {
                gacha_id: 3,
                expires_at: 200,
            }]
        );
        assert!(active(&pool, 200).await.unwrap().is_empty());

        enable(&pool, 3, 300).await.unwrap();
        assert!(disable(&pool, 3).await.unwrap());
        assert!(active(&pool, 100).await.unwrap().is_empty());
    }
}
