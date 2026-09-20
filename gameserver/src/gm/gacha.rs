use logic::{GachaAvailability, gacha_availability};
use muipserver::GmResponse;
use serde::Serialize;

use crate::net::app::AppState;

const OVERRIDE_DURATION_SECONDS: i32 = 30 * 86_400;

#[derive(Serialize)]
struct BannerCatalogEntry {
    id: i32,
    name: String,
    status: &'static str,
    naturally_active: bool,
    override_enabled: bool,
    can_override: bool,
    open_time: String,
    duration_days: i32,
    remaining_seconds: i32,
}

pub(super) async fn list(state: &AppState) -> GmResponse {
    let now = common::time::ServerTime::now_seconds_i32();
    let zone_offset = common::config().server.zone_offset;
    let overrides = match database::db::gacha_banner_overrides::active(&state.db, now).await {
        Ok(overrides) => overrides,
        Err(error) => return GmResponse::error(500, format!("could not load banners: {error}")),
    };
    let mut banners = state
        .tables
        .gacha_pools
        .rows
        .iter()
        .map(|pool| {
            let availability = gacha_availability(pool, now, zone_offset);
            let naturally_active = matches!(
                availability,
                GachaAvailability::Permanent | GachaAvailability::Active { .. }
            );
            let override_expires_at = overrides
                .iter()
                .find_map(|entry| (entry.gacha_id == pool.id).then_some(entry.expires_at));
            let override_enabled = override_expires_at.is_some();
            let status = match availability {
                GachaAvailability::Permanent => "permanent",
                GachaAvailability::Active { .. } => "active",
                _ if override_enabled => "enabled",
                GachaAvailability::Upcoming => "upcoming",
                GachaAvailability::Expired => "expired",
                GachaAvailability::InvalidSchedule => "invalid_schedule",
            };
            BannerCatalogEntry {
                id: pool.id,
                name: banner_name(state, pool),
                status,
                naturally_active,
                override_enabled,
                can_override: override_enabled
                    || matches!(
                        availability,
                        GachaAvailability::Upcoming | GachaAvailability::Expired
                    ),
                open_time: pool.open_time.clone(),
                duration_days: pool.duration,
                remaining_seconds: match availability {
                    GachaAvailability::Active { remaining_seconds } => remaining_seconds,
                    _ => override_expires_at.map_or(0, |expires_at| expires_at - now),
                },
            }
        })
        .collect::<Vec<_>>();
    banners.sort_by_key(|banner| banner.id);
    GmResponse {
        data: serde_json::to_value(banners).ok(),
        ..GmResponse::ok("gacha banners loaded")
    }
}

pub(super) async fn set_enabled(state: &AppState, gacha_id: i32, enabled: bool) -> GmResponse {
    let Some(pool) = state.tables.gacha_pools.get(gacha_id) else {
        return GmResponse::error(404, format!("unknown gacha banner {gacha_id}"));
    };
    if !enabled {
        return match database::db::gacha_banner_overrides::disable(&state.db, gacha_id).await {
            Ok(changed) => GmResponse {
                changed: usize::from(changed),
                ..GmResponse::ok(format!("banner {gacha_id} override disabled"))
            },
            Err(error) => GmResponse::error(500, format!("could not disable banner: {error}")),
        };
    }

    let now = common::time::ServerTime::now_seconds_i32();
    let availability = gacha_availability(pool, now, common::config().server.zone_offset);
    match availability {
        GachaAvailability::Permanent | GachaAvailability::Active { .. } => {
            return GmResponse::ok(format!(
                "banner {gacha_id} is already active from its table schedule"
            ));
        }
        GachaAvailability::InvalidSchedule => {
            return GmResponse::error(
                400,
                format!("banner {gacha_id} has an invalid table schedule"),
            );
        }
        GachaAvailability::Upcoming | GachaAvailability::Expired => {}
    }

    let expires_at = now.saturating_add(OVERRIDE_DURATION_SECONDS);
    match database::db::gacha_banner_overrides::enable(&state.db, gacha_id, expires_at).await {
        Ok(()) => GmResponse {
            changed: 1,
            ..GmResponse::ok(format!("banner {gacha_id} enabled for 30 days"))
        },
        Err(error) => GmResponse::error(500, format!("could not enable banner: {error}")),
    }
}

fn banner_name(state: &AppState, pool: &configs::tables::GachaPool) -> String {
    let names = pool
        .featured_details
        .iter()
        .map(|id| {
            state
                .tables
                .maids
                .get(*id)
                .map(|maid| (&maid.english_name, &maid.name))
                .or_else(|| {
                    state
                        .tables
                        .partners
                        .get(*id)
                        .map(|partner| (&partner.english_name, &partner.name))
                })
                .map_or_else(
                    || id.to_string(),
                    |(english, fallback)| {
                        if english.is_empty() {
                            fallback.clone()
                        } else {
                            english.clone()
                        }
                    },
                )
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        if pool.time_type == 0 {
            "Permanent banner".to_owned()
        } else {
            format!("Banner {}", pool.id)
        }
    } else {
        names.join(" / ")
    }
}
