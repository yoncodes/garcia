use muipserver::GmResponse;
use serde::Serialize;

use crate::net::app::AppState;

const OVERRIDE_DURATION_SECONDS: i32 = 30 * 86_400;

#[derive(Serialize)]
struct BossRushCatalogEntry {
    id: i32,
    name: String,
    status: &'static str,
    naturally_active: bool,
    override_enabled: bool,
    can_override: bool,
    open_time: String,
    close_time: String,
    remaining_seconds: i32,
}

pub(super) async fn list(state: &AppState) -> GmResponse {
    let now = common::time::ServerTime::now_seconds_i32();
    let zone_offset = common::config().server.zone_offset;
    let active_override = match database::db::boss_rush_override::active(&state.db, now).await {
        Ok(entry) => entry,
        Err(error) => return GmResponse::error(500, format!("could not load Boss Rush: {error}")),
    };
    let mut seasons = state
        .tables
        .boss_rushes
        .rows
        .iter()
        .map(|event| {
            let naturally_active = common::time::table_window_active(
                &event.open_time,
                &event.close_time,
                now,
                zone_offset,
            );
            let override_enabled = active_override.is_some_and(|entry| entry.event_id == event.id);
            let status = if naturally_active {
                "active"
            } else if override_enabled {
                "enabled"
            } else if common::time::table_time_utc(&event.open_time, zone_offset)
                .is_some_and(|open| open > i64::from(now))
            {
                "upcoming"
            } else {
                "expired"
            };
            BossRushCatalogEntry {
                id: event.id,
                name: format!("Boss Rush season {}", event.id),
                status,
                naturally_active,
                override_enabled,
                can_override: override_enabled || !naturally_active,
                open_time: event.open_time.clone(),
                close_time: event.close_time.clone(),
                remaining_seconds: if naturally_active {
                    common::time::table_window_remaining(
                        &event.open_time,
                        &event.close_time,
                        now,
                        zone_offset,
                    )
                    .unwrap_or_default()
                } else if override_enabled {
                    active_override.map_or(0, |entry| entry.expires_at - now)
                } else {
                    0
                },
            }
        })
        .collect::<Vec<_>>();
    seasons.sort_by_key(|event| event.id);
    GmResponse {
        data: serde_json::to_value(seasons).ok(),
        ..GmResponse::ok("Boss Rush seasons loaded")
    }
}

pub(super) async fn set_enabled(state: &AppState, event_id: i32, enabled: bool) -> GmResponse {
    let Some(event) = state.tables.boss_rushes.get(event_id) else {
        return GmResponse::error(404, format!("unknown Boss Rush season {event_id}"));
    };
    let now = common::time::ServerTime::now_seconds_i32();
    if !enabled {
        return match database::db::boss_rush_override::disable(&state.db, event_id, now).await {
            Ok(changed) => GmResponse {
                changed: usize::from(changed),
                ..GmResponse::ok(format!("Boss Rush season {event_id} override disabled"))
            },
            Err(error) => GmResponse::error(500, format!("could not disable Boss Rush: {error}")),
        };
    }

    if common::time::table_window_active(
        &event.open_time,
        &event.close_time,
        now,
        common::config().server.zone_offset,
    ) {
        return GmResponse::ok(format!(
            "Boss Rush season {event_id} is already active from its table schedule"
        ));
    }

    let expires_at = now.saturating_add(OVERRIDE_DURATION_SECONDS);
    match database::db::boss_rush_override::enable(&state.db, event_id, now, expires_at).await {
        Ok(()) => GmResponse {
            changed: 1,
            ..GmResponse::ok(format!("Boss Rush season {event_id} enabled for 30 days"))
        },
        Err(error) => GmResponse::error(500, format!("could not enable Boss Rush: {error}")),
    }
}
