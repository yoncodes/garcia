use super::*;

fn tables() -> GameTables {
    GameTables::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables"))
        .unwrap()
}

fn override_period(event_id: i32, expires_at: i32, instant_rewards: bool) -> BossRushOverride {
    BossRushOverride {
        event_id,
        started_at: expires_at - 60,
        expires_at,
        instant_rewards,
    }
}

#[test]
fn info_matches_captured_open_event() {
    let response = boss_rush_info(&tables(), 1_787_522_879, -25_200, None);
    assert!(response.open_state);
    assert_eq!(response.countdown, 46_320);
    assert_eq!(response.next_open_countdown, 0);
    assert_eq!(response.close_time, 1_787_569_199);

    let info = response.boss_info.unwrap();
    assert_eq!((info.curr_bid, info.last_bid), (1005, 1005));
    assert!(info.boss_base.is_empty());
    assert!(info.ids.is_empty());
    assert_eq!(info.max_damage, 0);
}

#[test]
fn expired_season_override_reopens_boss_rush() {
    let tables = tables();
    let now = 1_789_943_200;
    let response = boss_rush_info(
        &tables,
        now,
        -14_400,
        Some(override_period(1005, now + 2_592_000, false)),
    );
    assert!(response.open_state);
    assert_eq!(response.countdown, 2_592_000);
    assert_eq!(response.boss_info.unwrap().curr_bid, 1005);
}

#[test]
fn older_overridden_season_remains_the_current_season() {
    let tables = tables();
    let now = 1_789_943_200;
    let response = boss_rush_info(
        &tables,
        now,
        -14_400,
        Some(override_period(1001, now + 60, false)),
    );
    let info = response.boss_info.unwrap();
    assert_eq!((info.curr_bid, info.last_bid), (1001, 1005));
}

#[test]
fn closed_override_remains_visible_during_reward_delay() {
    let tables = tables();
    let close = 1_789_943_200;
    let period = override_period(1001, close, false);
    let response = boss_rush_info(&tables, close + 1, -14_400, Some(period));
    let info = response.boss_info.unwrap();
    assert!(!response.open_state);
    assert_eq!((info.curr_bid, info.last_bid), (0, 1001));
    assert_eq!(response.close_time, i64::from(close));
    assert_eq!(
        ranking_reward_ready_at(&tables, 1001, -14_400, Some(period)),
        Some(close + 24 * 60 * 60)
    );
    assert_eq!(
        ranking_reward_ready_at(
            &tables,
            1001,
            -14_400,
            Some(override_period(1001, close, true)),
        ),
        Some(close)
    );
}

#[test]
fn reset_team_removes_only_the_requested_port() {
    let tables = tables();
    let event = active_boss_rush(&tables, 1_787_522_879, -25_200).unwrap();
    let config = common::load_config().unwrap();
    let mut player = crate::logic::Player::new(42, &tables, &config.account_defaults);
    let first = event.port_id[0];
    let second = event.port_id[1];
    player
        .settle_boss_rush(event, first, 100, Vec::new())
        .unwrap();
    player
        .settle_boss_rush(event, second, 200, Vec::new())
        .unwrap();

    let remaining = player.reset_boss_rush_team(event, first).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!((remaining[0].pid, remaining[0].damage), (second, 200));
}
