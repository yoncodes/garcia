use super::*;

pub(super) fn consume_item_costs<E>(
    inventory: &mut [DcNetDataItem],
    costs: &BTreeMap<i32, i32>,
    error: impl Fn(i32, i32, i32) -> E,
) -> Result<Vec<DcNetDataItem>, E> {
    for (&item_id, &needed) in costs {
        let available = inventory
            .iter()
            .find(|item| item.item_id == item_id)
            .map_or(0, |item| item.amount);
        if needed <= 0 || available < needed {
            return Err(error(item_id, needed, available));
        }
    }
    let mut remains = Vec::with_capacity(costs.len());
    for (&item_id, &amount) in costs {
        let item = inventory
            .iter_mut()
            .find(|item| item.item_id == item_id)
            .unwrap();
        item.amount -= amount;
        remains.push(*item);
    }
    Ok(remains)
}

pub(super) fn push_red_dot(reddots: &mut Vec<DcNetDataRedDot>, func: Reddot, arg: impl ToString) {
    let arg = arg.to_string();
    let id = format!("{}_{}", func as i32, arg);
    if reddots.iter().any(|red_dot| red_dot.id == id) {
        return;
    }
    reddots.push(DcNetDataRedDot {
        arg,
        func_id: func as i32,
        id,
        ..Default::default()
    });
}

pub(super) fn is_mail_active(state: &MailState, now: i32) -> bool {
    state.expires_at == 0 || state.expires_at > now
}

pub(super) fn role_level_limit(rank: i32, tables: &GameTables) -> i32 {
    let mut limit = 1;
    for level in &tables.maid_levels.rows {
        limit = level.level;
        if level.required_rank > rank {
            break;
        }
    }
    limit
}

pub(super) fn visible_battle_pass_tasks<'a>(
    config: &configs::tables::BattlePass,
    state: &BattlePassState,
    tables: &'a GameTables,
) -> Vec<&'a configs::tables::BattlePassTask> {
    let mut chains = BTreeMap::<i32, Vec<&configs::tables::BattlePassTask>>::new();
    for task in tables
        .battle_pass_tasks
        .rows
        .iter()
        .filter(|task| task.bp_id == config.id)
    {
        chains.entry(task.chain).or_default().push(task);
    }
    chains
        .into_values()
        .filter_map(|mut tasks| {
            tasks.sort_by_key(|task| task.chain_order);
            tasks
                .iter()
                .find(|task| !state.tasks.contains(&task.id))
                .copied()
                .or_else(|| tasks.last().copied())
        })
        .collect()
}

pub(super) fn condition_total(condition: i32, value: &str) -> i32 {
    if matches!(condition, 81 | 85) {
        1
    } else if matches!(condition, 25 | 26) {
        value
            .split('|')
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or_default()
    } else {
        value
            .rsplit('|')
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or_default()
    }
}

pub(super) fn activity_seconds_remaining(
    activity: &configs::tables::Activity,
    now: i32,
    zone_offset: i32,
) -> Option<i32> {
    let now = i64::from(now);
    match activity.time_type {
        0 | 1 => Some(0),
        3 => {
            let start = common::time::table_time_utc(&activity.open_time, zone_offset)?;
            (now >= start).then_some(0)
        }
        4 => {
            let start = common::time::table_time_utc(&activity.open_time, zone_offset)?;
            let end = common::time::table_time_utc(&activity.close_time, zone_offset)?;
            (now >= start && now < end).then(|| (end - now).min(i64::from(i32::MAX)) as i32)
        }
        _ => None,
    }
}

pub(super) fn draw_seconds_remaining(
    config: &configs::tables::GachaPool,
    now: i32,
    zone_offset: i32,
) -> Option<i32> {
    match gacha_availability(config, now, zone_offset) {
        GachaAvailability::Permanent => Some(0),
        GachaAvailability::Active { remaining_seconds } => Some(remaining_seconds),
        _ => None,
    }
}

pub fn gacha_availability(
    config: &configs::tables::GachaPool,
    now: i32,
    zone_offset: i32,
) -> GachaAvailability {
    if config.time_type == 0 {
        return GachaAvailability::Permanent;
    }
    let Some(start) = common::time::table_time_utc(&config.open_time, zone_offset) else {
        return GachaAvailability::InvalidSchedule;
    };
    let end = start.saturating_add(i64::from(config.duration).saturating_mul(86_400));
    let now = i64::from(now);
    if now < start {
        GachaAvailability::Upcoming
    } else if now >= end {
        GachaAvailability::Expired
    } else {
        GachaAvailability::Active {
            remaining_seconds: (end - now).min(i64::from(i32::MAX)) as i32,
        }
    }
}

pub(super) fn gacha_seconds_remaining(
    config: &configs::tables::GachaPool,
    now: i32,
    zone_offset: i32,
    override_expires_at: Option<i32>,
) -> Option<i32> {
    draw_seconds_remaining(config, now, zone_offset).or_else(|| {
        override_expires_at
            .filter(|expires_at| *expires_at > now)
            .map(|expires_at| expires_at - now)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GachaCategory {
    Quality6Maid,
    Quality5Maid,
    Quality5Partner,
    Quality4Partner,
}

const FEATURED_GACHA_WEIGHT: i32 = 5_000;

pub(super) fn select_gacha_category(
    pool: &configs::tables::GachaPool,
    guaranteed: bool,
    tables: &GameTables,
    mut roll: i32,
) -> Option<GachaCategory> {
    let weights = tables.gacha_weights.get(pool.id)?;
    let weight = |quality| {
        weights
            .rarity
            .iter()
            .find(|entry| entry.key == quality)
            .map_or(0, |entry| entry.value.max(0))
    };
    let quality6 = weight(6);
    let quality5 = weight(5);
    let quality4 = weight(4);
    let total = quality6.saturating_add(quality5).saturating_add(quality4);
    if total <= 0 {
        return None;
    }
    roll = roll.rem_euclid(total);

    if roll < quality6 {
        return Some(GachaCategory::Quality6Maid);
    }
    roll -= quality6;
    if guaranteed {
        return Some(GachaCategory::Quality5Maid);
    }

    // Published rules split the 10% quality-5 rate evenly between Companions
    // and Familiars; the rarity table stores only their combined weight.
    let quality5_maids = quality5 / 2;
    if roll < quality5_maids {
        return Some(GachaCategory::Quality5Maid);
    }
    roll -= quality5_maids;
    if roll < quality5.saturating_sub(quality5_maids) {
        return Some(GachaCategory::Quality5Partner);
    }
    Some(GachaCategory::Quality4Partner)
}

pub(super) fn select_gacha_reward(
    pool: &configs::tables::GachaPool,
    guaranteed: bool,
    tables: &GameTables,
) -> Option<i32> {
    let weights = tables.gacha_weights.get(pool.id)?;
    let total_weight: i32 = weights.rarity.iter().map(|entry| entry.value.max(0)).sum();
    if total_weight <= 0 {
        return None;
    }
    let category = select_gacha_category(pool, guaranteed, tables, fastrand::i32(0..total_weight))?;
    let reward_id = if guaranteed {
        pool.ten_reward_id
    } else {
        pool.reward_id
    };
    let mut rewards = Vec::new();
    collect_reward_leaves(reward_id, tables, &mut rewards, 0);
    let candidates: Vec<_> = rewards
        .into_iter()
        .filter(|id| match category {
            GachaCategory::Quality6Maid => {
                tables.maids.get(*id).is_some_and(|maid| maid.quality == 6)
            }
            GachaCategory::Quality5Maid => {
                tables.maids.get(*id).is_some_and(|maid| maid.quality == 5)
            }
            GachaCategory::Quality5Partner => tables
                .partners
                .get(*id)
                .is_some_and(|partner| partner.quality == 5),
            GachaCategory::Quality4Partner => tables
                .partners
                .get(*id)
                .is_some_and(|partner| partner.quality == 4),
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }

    select_gacha_candidate(
        pool,
        &candidates,
        fastrand::i32(0..10_000),
        fastrand::usize(..usize::MAX),
    )
}

pub(super) fn select_gacha_candidate(
    pool: &configs::tables::GachaPool,
    candidates: &[i32],
    featured_roll: i32,
    candidate_roll: usize,
) -> Option<i32> {
    if candidates.is_empty() {
        return None;
    }
    if pool.pool_type != 2 {
        return Some(candidates[candidate_roll % candidates.len()]);
    }

    let featured: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|id| pool.featured_details.contains(id))
        .collect();
    // The client-facing banner rules specify a 50% featured split. The
    // extracted `up_weight` value is not the displayed probability.
    if !featured.is_empty() && featured_roll.rem_euclid(10_000) < FEATURED_GACHA_WEIGHT {
        return Some(featured[candidate_roll % featured.len()]);
    }
    let regular: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|id| !featured.contains(id))
        .collect();
    if regular.is_empty() {
        return Some(candidates[candidate_roll % candidates.len()]);
    }
    Some(regular[candidate_roll % regular.len()])
}

pub(super) fn select_reward(reward_id: i32, tables: &GameTables) -> Option<i32> {
    let mut rewards = Vec::new();
    collect_reward_leaves(reward_id, tables, &mut rewards, 0);
    if rewards.is_empty() {
        return None;
    }
    rewards.get(fastrand::usize(..rewards.len())).copied()
}

pub(super) fn collect_reward_leaves(
    reward_id: i32,
    tables: &GameTables,
    output: &mut Vec<i32>,
    depth: usize,
) {
    if depth >= 8 {
        return;
    }
    let Some(config) = tables.rewards.get(reward_id) else {
        output.push(reward_id);
        return;
    };
    for reward in &config.reward {
        for _ in 0..reward.value.max(0) {
            collect_reward_leaves(reward.key, tables, output, depth + 1);
        }
    }
}

pub(super) fn gacha_role(
    uid: i64,
    id: i32,
    tables: &GameTables,
) -> Option<DcNetDataRolesRolesDetail> {
    let maid = tables.maids.get(id)?;
    Some(DcNetDataRolesRolesDetail {
        role_basic_info: Some(DcNetDataRoleBasicInfo {
            game_role_id: id,
            level: 1,
            qua: maid.quality,
            element: maid.element,
            appear_skill_key: 2,
            element4call: maid.element,
            user_role_id: make_entity_id(uid, id),
            ..Default::default()
        }),
        talents: tables
            .initial_talents(id)
            .map(|(position, lv)| DcNetDataTalent {
                game_role_id: id,
                position,
                lv,
            })
            .collect(),
        ..Default::default()
    })
}

pub(super) fn initial_daily_tasks(
    uid: i64,
    day: i32,
    tables: &GameTables,
) -> Vec<DcNetDataTaskCycle> {
    let mut selected = Vec::new();
    let mut groups = BTreeMap::<i32, Vec<&configs::tables::DailyTaskDefinition>>::new();
    for task in &tables.daily_tasks.rows {
        if task.group == 0 {
            selected.push(task);
        } else {
            groups.entry(task.group).or_default().push(task);
        }
    }
    for (group, mut tasks) in groups {
        tasks.sort_by_key(|task| task.id);
        let count = tasks[0].group_selet.max(0) as usize;
        let offset =
            ((uid as u64) ^ (day as u64).rotate_left(17) ^ group as u64) as usize % tasks.len();
        selected
            .extend((0..count.min(tasks.len())).map(|index| tasks[(offset + index) % tasks.len()]));
    }
    let mut result: Vec<_> = selected
        .into_iter()
        .filter_map(|task| {
            let limit = task.finish_limit.first()?;
            let total = limit.value.rsplit('|').next()?.parse().ok()?;
            Some(DcNetDataTaskCycle {
                id: task.id,
                progress: if limit.key == 4 { total } else { 0 },
                total,
                ..Default::default()
            })
        })
        .collect();
    result.sort_by_key(|task| task.id);
    result
}

pub(super) fn starter_role(
    uid: i64,
    id: i32,
    tables: &GameTables,
    defaults: &AccountDefaults,
) -> Option<DcNetDataRolesRolesDetail> {
    let maid = tables.maids.get(id)?;
    Some(DcNetDataRolesRolesDetail {
        role_basic_info: Some(DcNetDataRoleBasicInfo {
            game_role_id: id,
            level: defaults.level,
            qua: maid.quality,
            element: maid.element,
            appear_skill_key: defaults.appear_skill_key,
            element4call: maid.element,
            user_role_id: make_entity_id(uid, id),
            cur_mc: id == tables.default_main_maid(),
            ..Default::default()
        }),
        talents: tables
            .initial_talents(id)
            .map(|(position, lv)| DcNetDataTalent {
                game_role_id: id,
                position,
                lv,
            })
            .collect(),
        ..Default::default()
    })
}

pub(super) fn make_entity_id(uid: i64, id: i32) -> i64 {
    uid.wrapping_mul(1_000_000_000).wrapping_add(i64::from(id))
}
