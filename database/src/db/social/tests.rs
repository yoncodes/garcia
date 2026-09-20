use super::*;

async fn add_player(pool: &SqlitePool, uid: i64) {
    sqlx::query(
        "INSERT INTO players (
            uid, username, nickname, level, gameplay_id, created_at, last_login_time,
            cur_form, profile_avatar, profile_card, profile_title, profile_frame, region,
            banner_girl, traced_task_group, heat_updated_at
         ) VALUES (?, '', '', 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
    )
    .bind(uid)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn friendship_and_daily_gifts_are_shared_and_persistent() {
    let pool = crate::connect_memory().await.unwrap();
    add_player(&pool, 10).await;
    add_player(&pool, 20).await;

    assert!(send_friend_request(&pool, 10, 20, 100).await.unwrap());
    assert_eq!(
        load_state(&pool, 20, 7).await.unwrap().pending_approvals,
        vec![10]
    );
    assert!(accept_friend_request(&pool, 20, 10, 50, 101).await.unwrap());
    assert_eq!(load_state(&pool, 10, 7).await.unwrap().friends[0].uid, 20);
    assert_eq!(load_state(&pool, 20, 7).await.unwrap().friends[0].uid, 10);

    assert!(send_gift(&pool, 10, 20, 7).await.unwrap());
    assert!(!send_gift(&pool, 10, 20, 7).await.unwrap());
    let recipient = load_state(&pool, 20, 7).await.unwrap();
    assert!(recipient.friends[0].has_gift);
    assert_eq!(claim_gift(&pool, 20, 10, 7, 10).await.unwrap(), Some(1));
    assert_eq!(claim_gift(&pool, 20, 10, 7, 10).await.unwrap(), None);
    assert_eq!(load_state(&pool, 20, 7).await.unwrap().claimed_gifts, 1);

    assert!(remove_friend(&pool, 10, 20).await.unwrap());
    assert!(load_state(&pool, 10, 7).await.unwrap().friends.is_empty());
    assert!(load_state(&pool, 20, 7).await.unwrap().friends.is_empty());
}
