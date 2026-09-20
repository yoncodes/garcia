use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
    tables: &GameTables,
) {
    if record.daily_day == common::time::day_index(ServerTime::now_seconds_i32())
        && !record.daily_tasks.is_empty()
    {
        player.daily_day = std::mem::take(&mut record.daily_day);
        player.daily_activity = std::mem::take(&mut record.daily_activity);
        player.daily_reward_progress = std::mem::take(&mut record.daily_reward_progress);
        player.daily_tasks = std::mem::take(&mut record.daily_tasks)
            .into_iter()
            .map(|task| DcNetDataTaskCycle {
                id: task.id,
                taken: task.taken,
                progress: task.progress,
                total: task.total,
            })
            .collect();
    }
    player.heat_exchange_day = std::mem::take(&mut record.heat_exchange_day);
    player.heat_exchange_count = std::mem::take(&mut record.heat_exchange_count);
    player.gold_coins = std::mem::take(&mut record.gold_coins)
        .into_iter()
        .map(|coin| CollectedGoldCoin {
            city_id: coin.city_id,
            coin_id: coin.coin_id,
        })
        .collect();
    player.collection_resources = std::mem::take(&mut record.collection_resources)
        .into_iter()
        .map(|resource| CollectionResourceState {
            collection_id: resource.collection_id,
            remaining: resource.remaining,
        })
        .collect();
    let mut monster_manuals = BTreeMap::<i32, Vec<u32>>::new();
    for monster in std::mem::take(&mut record.monster_manuals) {
        if !tables
            .monsters
            .rows
            .iter()
            .any(|config| config.id_crc == monster.enemy_hash)
        {
            continue;
        }
        let hashes = monster_manuals.entry(monster.gameplay_id).or_default();
        if !hashes.contains(&monster.enemy_hash) {
            hashes.push(monster.enemy_hash);
        }
    }
    player.monster_manuals = monster_manuals
        .into_iter()
        .map(|(port_id, mons)| DcNetDataMonsterManual4Port { port_id, mons })
        .collect();
    player.defeated_monster_points = std::mem::take(&mut record.monster_points)
        .into_iter()
        .filter(|point| point.day == player.wild_monster_day)
        .map(|point| point.point_id)
        .collect();
    player.region_coin_daily = std::mem::take(&mut record.region_coin_daily)
        .into_iter()
        .filter(|coin| coin.day == player.wild_monster_day)
        .map(|coin| (coin.coin_id, coin.amount))
        .collect();
    player.region_ticket_day = std::mem::take(&mut record.region_ticket_day);
    player.region_levels = std::mem::take(&mut record.region_levels)
        .into_iter()
        .map(|region| (region.region_id, region.level))
        .collect();
    player.reward_boxes = std::mem::take(&mut record.reward_boxes)
        .into_iter()
        .map(|reward_box| DcNetDataTBoxInfo {
            id: reward_box.id,
            status: reward_box.status,
        })
        .collect();
}
