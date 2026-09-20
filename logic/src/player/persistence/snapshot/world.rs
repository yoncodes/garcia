use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.tp_map = player
        .tp_map
        .iter()
        .map(
            |(key, value)| database::models::game::player_state::TpMapRecord {
                key: key.clone(),
                value: value.clone(),
            },
        )
        .collect();
    record.role_attrs = player
        .role_attrs
        .iter()
        .map(
            |attr| database::models::game::player_state::RoleAttrRecord {
                role_id: attr.role_id,
                mp: attr.mp,
                ep: attr.ep,
                hp: attr.hp,
            },
        )
        .collect();
    record.playing_port = player.playing_port.as_ref().map(|port| {
        database::models::game::player_state::PlayingPortRecord {
            port_index_id: port.port_index_id,
            port_info: port.port_info.clone(),
        }
    });
    record.interact_objs = player
        .interact_objs
        .iter()
        .map(|object| database::models::game::world::InteractRecord {
            object_id: object.object_id.clone(),
            interactive: object.interactive,
            status: object.status,
            count: object.count,
        })
        .collect();
    record.challenges = player
        .challenges
        .iter()
        .map(|challenge| database::models::game::world::ChallengeRecord {
            id: challenge.id.clone(),
            stars: challenge.stars,
            finished: challenge.finished,
            claimed: challenge.claimed,
        })
        .collect();
    record.albums = player
        .albums
        .iter()
        .map(|album| database::models::game::world::AlbumRecord {
            id: album.id,
            status: album.status,
            sort: album.sort,
        })
        .collect();
    record.dense_fogs = player.dense_fogs.clone();
    record.ports = player
        .ports
        .iter()
        .map(|port| database::models::game::player_state::PortRecord {
            id: port.id,
            pass_cnt: port.pass_cnt,
            is_c: port.is_c,
            is_f: port.is_f,
            port_id: port.port_id,
            s1: port.s1,
            s2: port.s2,
            s3: port.s3,
            t_cnt: port.t_cnt,
            updated_at: port.updated_at,
            all_cnt: port.all_cnt,
        })
        .collect();
    record.completed_dungeons = player.completed_dungeons.clone();
    record.dungeon_clears = player
        .dungeon_clears
        .iter()
        .map(
            |(&dungeon_type, &count)| database::models::game::player_state::DungeonClearRecord {
                dungeon_type,
                count,
            },
        )
        .collect();
    record.gold_coins = player
        .gold_coins
        .iter()
        .map(
            |coin| database::models::game::player_state::GoldCoinRecord {
                city_id: coin.city_id,
                coin_id: coin.coin_id.clone(),
            },
        )
        .collect();
    record.collection_resources = player
        .collection_resources
        .iter()
        .map(
            |resource| database::models::game::player_state::CollectionResourceRecord {
                collection_id: resource.collection_id.clone(),
                remaining: resource.remaining,
            },
        )
        .collect();
    record.monster_points = player
        .defeated_monster_points
        .iter()
        .map(
            |point_id| database::models::game::player_state::MonsterPointRecord {
                day: player.wild_monster_day,
                point_id: point_id.clone(),
            },
        )
        .collect();
    record.monster_manuals = player
        .monster_manuals
        .iter()
        .flat_map(|manual| {
            manual.mons.iter().map(|&enemy_hash| {
                database::models::game::player_state::MonsterManualRecord {
                    gameplay_id: manual.port_id,
                    enemy_hash,
                }
            })
        })
        .collect();
    record.region_coin_daily = player
        .region_coin_daily
        .iter()
        .map(
            |(&coin_id, &amount)| database::models::game::player_state::RegionCoinDailyRecord {
                day: player.wild_monster_day,
                coin_id,
                amount,
            },
        )
        .collect();
    record.region_ticket_day = player.region_ticket_day;
    record.region_levels =
        player
            .region_levels
            .iter()
            .map(|(&region_id, &level)| {
                database::models::game::player_state::RegionProgressRecord { region_id, level }
            })
            .collect();
    record.reward_boxes = player
        .reward_boxes
        .iter()
        .map(
            |reward_box| database::models::game::player_state::RewardBoxRecord {
                id: reward_box.id.clone(),
                status: reward_box.status,
            },
        )
        .collect();
}
