use super::*;
use crate::net::gm_command::GmCommand;
use common::network::crypto::{DH_BASE, DH_PRIME, KEY_SALT, Rc4, crypt_packet_body, mod_pow};
use configs::GameTables;
use protocol::{
    cs::{DcNetWorkingNotifyRedeemGiftCode, DcNetWorkingNotifyTaskCycle},
    pbcommon::DcNetDataPhy,
};

#[tokio::test]
async fn encrypted_login_and_gm_reward_pushes_succeed() {
    let client_send_secret = 7;
    let client_receive_secret = 11;
    let seed_request = C2gGetSeed {
        send_seed: mod_pow(DH_BASE, client_send_secret, DH_PRIME) as i32,
        receive_seed: mod_pow(DH_BASE, client_receive_secret, DH_PRIME) as i32,
    };

    let config = common::load_config().unwrap();
    common::init_config(config.clone());
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let database = database::connect_memory().await.unwrap();
    let state = Arc::new(AppState::new(database.clone(), tables));
    let (gm_tx, _gm_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut gateway = GatewaySession::new(state.clone(), gm_tx);

    let seed_response = gateway
        .handle_packet(&client_packet(1, Id::C2gGetSeedId, seed_request))
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let seed_response = ServerPacket::decode(&seed_response).unwrap();
    let seeds = G2cGetSeed::decode(seed_response.payload.as_slice()).unwrap();

    let mut client_send = Rc4::new(
        format!(
            "{KEY_SALT}{}",
            mod_pow(seeds.send_seed as i64, client_send_secret, DH_PRIME)
        )
        .as_bytes(),
    );
    let mut client_receive = Rc4::new(
        format!(
            "{KEY_SALT}{}",
            mod_pow(seeds.receive_seed as i64, client_receive_secret, DH_PRIME)
        )
        .as_bytes(),
    );

    let mut login = client_packet(
        2,
        Id::C2gConnectGameServerId,
        C2gConnectGameServer {
            uid: 42,
            sid: "sid".into(),
            token: "token".into(),
        },
    );
    crypt_packet_body(&mut login, &mut client_send).unwrap();
    let mut login_packets = gateway.handle_packet(&login).await.unwrap();
    assert_eq!(login_packets.len(), 3);
    for packet in &mut login_packets {
        crypt_packet_body(packet, &mut client_receive).unwrap();
    }
    let response = ServerPacket::decode(&login_packets[0]).unwrap();

    assert_eq!(response.proto_id, Id::G2cConnectGameServerId as u16);
    assert_eq!(
        G2cConnectGameServer::decode(response.payload.as_slice())
            .unwrap()
            .ret,
        1
    );
    let daily = ServerPacket::decode(&login_packets[1]).unwrap();
    assert_eq!(daily.proto_id, NotifyId::DcNetWorkingNotifyTaskCycle as u16);
    let daily = DcNetWorkingNotifyTaskCycle::decode(daily.payload.as_slice()).unwrap();
    assert_eq!(daily.task.unwrap().id, 1);

    let heat = ServerPacket::decode(&login_packets[2]).unwrap();
    assert_eq!(heat.proto_id, NotifyId::DcNetDataPhy as u16);
    let heat = DcNetDataPhy::decode(heat.payload.as_slice()).unwrap();
    assert_eq!(
        heat.item.unwrap().item_id,
        state.tables.cultivation_constants.heat_item_id
    );
    assert_eq!(gateway.player_uid(), Some(42));

    let (_, mut world_level_packets) = gateway
        .handle_gm_command(GmCommand::IncreaseWorldLevel(1))
        .await
        .unwrap();
    for packet in &mut world_level_packets {
        crypt_packet_body(packet, &mut client_receive).unwrap();
    }
    assert!(world_level_packets.iter().any(|packet| {
        ServerPacket::decode(packet)
            .is_ok_and(|packet| packet.proto_id == NotifyId::DcNetWorkingNotifyWorldLevel as u16)
    }));

    let (unchanged, mut resync_packets) = gateway
        .handle_gm_command(GmCommand::IncreaseWorldLevel(1))
        .await
        .unwrap();
    assert_eq!(unchanged.changed, 0);
    for packet in &mut resync_packets {
        crypt_packet_body(packet, &mut client_receive).unwrap();
    }
    assert_eq!(resync_packets.len(), 1);
    let resync = ServerPacket::decode(&resync_packets[0]).unwrap();
    assert_eq!(
        resync.proto_id,
        NotifyId::DcNetWorkingNotifyWorldLevel as u16
    );
    let resync =
        protocol::cs::DcNetWorkingNotifyWorldLevel::decode(resync.payload.as_slice()).unwrap();
    assert_eq!(resync.world_level, 1);
    let level = resync.updata.unwrap();
    assert_eq!(level.attr_lv, 20);
    assert!(level.is_lv_up);
    assert_eq!(level.items.len(), 1);

    let now = common::time::ServerTime::now_seconds_i32();
    let interval = state
        .tables
        .cultivation_constants
        .heat_regeneration_interval;
    let heat_item_id = state.tables.cultivation_constants.heat_item_id;
    {
        let mut player = gateway.context.update_player().unwrap();
        let heat = player
            .items
            .iter_mut()
            .find(|item| item.item_id == heat_item_id)
            .unwrap();
        heat.amount -= 1;
        player.heat_updated_at = now - interval;
    }
    gateway.last_heat_poll = now - 1;
    let mut periodic = gateway.poll_periodic().await.unwrap();
    assert_eq!(periodic.len(), 1);
    crypt_packet_body(&mut periodic[0], &mut client_receive).unwrap();
    let periodic = ServerPacket::decode(&periodic[0]).unwrap();
    assert_eq!(periodic.proto_id, NotifyId::DcNetDataPhy as u16);
    let periodic = DcNetDataPhy::decode(periodic.payload.as_slice()).unwrap();
    assert_eq!(periodic.item.unwrap().item_id, heat_item_id);

    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::UnlockFeatures)
        .await
        .unwrap();
    assert!(outcome.changed > 0);
    let mut pushed_ids = Vec::new();
    for push in &mut pushes {
        crypt_packet_body(push, &mut client_receive).unwrap();
        pushed_ids.push(ServerPacket::decode(push).unwrap().proto_id);
    }
    assert_eq!(
        pushed_ids.first().copied(),
        Some(NotifyId::DcNetDataTaskChanged as u16)
    );
    assert!(pushed_ids.contains(&(NotifyId::DcNetDataFeat as u16)));

    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::CompleteTutorials)
        .await
        .unwrap();
    assert!(outcome.changed > 0);
    assert!(!outcome.reconnect_required);
    assert_eq!(pushes.len(), 1);
    crypt_packet_body(&mut pushes[0], &mut client_receive).unwrap();
    let push = ServerPacket::decode(&pushes[0]).unwrap();
    assert_eq!(
        push.proto_id,
        crate::handlers::guide::TUTORIAL_REFRESH_NOTIFY_ID
    );
    let guides = protocol::cs::DcNetWorkingResUserGuideList::decode(push.payload.as_slice())
        .unwrap()
        .gids;
    assert_eq!(guides.len(), outcome.changed);

    let (collection, pushes) = gateway
        .handle_gm_command(GmCommand::Collection)
        .await
        .unwrap();
    assert!(pushes.is_empty());
    let collection = collection.data.unwrap();
    assert!(collection["heroes"].as_array().is_some_and(|rows| {
        rows.iter().any(|row| row["owned"] == true) && rows.iter().any(|row| row["owned"] == false)
    }));
    assert!(collection["pets"].as_array().is_some_and(|rows| {
        rows.iter()
            .any(|row| row["id"] == 100_461_900 && row["name"] == "Elder of Foolish Plays")
    }));
    assert!(collection["items"].as_array().is_some_and(|rows| {
        rows.iter().any(|row| {
            row["id"] == state.tables.cultivation_constants.diamond_item_id
                && row["name"] == "Emberite"
                && row["amount"].as_i64().is_some()
        })
    }));
    assert!(collection["relics"].as_array().is_some_and(|rows| {
        let blue_is_table_driven = rows.iter().any(|row| {
            row["id"] == 100_500_114
                && row["name"] == "Hero's Vinebound Clock · Blue · Slot 1"
                && row["amount"].as_i64().is_some()
                && row["substat_count"] == 2
                && row["main_stats"].as_array().is_some_and(|stats| {
                    stats
                        .iter()
                        .any(|stat| stat["name"] == "HP" && stat["value"].as_f64().is_some())
                })
                && row["substats"]
                    .as_array()
                    .is_some_and(|stats| stats.iter().any(|stat| stat["name"] == "Critical Rate"))
        });
        let gold_uses_all_six_rolls = rows
            .iter()
            .any(|row| row["id"] == 100_500_116 && row["substat_count"] == 6);
        let unfinished_relics_are_hidden = rows.iter().all(|row| row["id"] != 100_500_846);
        blue_is_table_driven && gold_uses_all_six_rolls && unfinished_relics_are_hidden
    }));

    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::GrantHeroes)
        .await
        .unwrap();
    assert!(outcome.changed > 0);
    assert!(!outcome.reconnect_required);
    assert_eq!(pushes.len(), 1);

    crypt_packet_body(&mut pushes[0], &mut client_receive).unwrap();
    let push = ServerPacket::decode(&pushes[0]).unwrap();
    assert_eq!(
        push.proto_id,
        NotifyId::DcNetWorkingNotifyRedeemGiftCode as u16
    );
    let reward = DcNetWorkingNotifyRedeemGiftCode::decode(push.payload.as_slice())
        .unwrap()
        .reward
        .unwrap();
    assert_eq!(reward.role_rewards.len(), outcome.changed);

    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::GrantPets)
        .await
        .unwrap();
    assert!(outcome.changed > 0);
    assert!(!outcome.reconnect_required);
    assert_eq!(pushes.len(), 1);

    crypt_packet_body(&mut pushes[0], &mut client_receive).unwrap();
    let push = ServerPacket::decode(&pushes[0]).unwrap();
    assert_eq!(
        push.proto_id,
        NotifyId::DcNetWorkingNotifyRedeemGiftCode as u16
    );
    let reward = DcNetWorkingNotifyRedeemGiftCode::decode(push.payload.as_slice())
        .unwrap()
        .reward
        .unwrap();
    assert_eq!(reward.partner_rewards.len(), outcome.changed);

    let item_id = state.tables.cultivation_constants.diamond_item_id;
    let before = gateway
        .context
        .player()
        .unwrap()
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .map_or(0, |item| item.amount);
    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::GrantItem(item_id, 500))
        .await
        .unwrap();
    assert_eq!(outcome.changed, 1);
    assert_eq!(pushes.len(), 1);
    crypt_packet_body(&mut pushes[0], &mut client_receive).unwrap();
    let push = ServerPacket::decode(&pushes[0]).unwrap();
    assert_eq!(
        push.proto_id,
        NotifyId::DcNetWorkingNotifyRedeemGiftCode as u16
    );
    let reward = DcNetWorkingNotifyRedeemGiftCode::decode(push.payload.as_slice())
        .unwrap()
        .reward
        .unwrap();
    assert_eq!(
        (reward.items[0].item_id, reward.items[0].amount),
        (item_id, before + 500)
    );
    let saved = database::db::player_state::load(&database, 42)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .unwrap()
            .amount,
        before + 500
    );

    let relic_id = 100_500_116;
    let relic = state.tables.equipment.get(relic_id).unwrap();
    let main_stat = state
        .tables
        .main_equipment_words_by_group
        .get(relic.group_id)
        .unwrap()
        .next()
        .unwrap()
        .id;
    let substats = state
        .tables
        .extra_equipment_words_by_slot
        .get(relic.slot)
        .unwrap()
        .take(6)
        .map(|word| word.id)
        .collect::<Vec<_>>();
    let relics_before = gateway
        .context
        .player()
        .unwrap()
        .equips
        .iter()
        .filter(|relic| relic.equip_id == relic_id)
        .count();
    let (outcome, mut pushes) = gateway
        .handle_gm_command(GmCommand::GenerateRelic {
            id: relic_id,
            amount: 2,
            main_stat,
            substats: substats.clone(),
        })
        .await
        .unwrap();
    assert_eq!(outcome.changed, 2);
    assert_eq!(pushes.len(), 1);
    crypt_packet_body(&mut pushes[0], &mut client_receive).unwrap();
    let push = ServerPacket::decode(&pushes[0]).unwrap();
    let reward = DcNetWorkingNotifyRedeemGiftCode::decode(push.payload.as_slice())
        .unwrap()
        .reward
        .unwrap();
    assert_eq!(reward.equips.len(), 2);
    assert!(reward.equips.iter().all(|relic| {
        relic.equip_id == relic_id
            && relic.main_words_id == main_stat
            && relic.deputy_words_id == substats
            && relic.random_words_id.is_empty()
    }));
    let saved = database::db::player_state::load(&database, 42)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved
            .equips
            .iter()
            .filter(|relic| relic.equip_id == relic_id)
            .count(),
        relics_before + 2
    );
    assert!(
        saved
            .equips
            .iter()
            .filter(|relic| relic.equip_id == relic_id)
            .all(|relic| relic.planned_words_id.is_empty())
    );

    let (collection, pushes) = gateway
        .handle_gm_command(GmCommand::Collection)
        .await
        .unwrap();
    assert!(pushes.is_empty());
    let collection = collection.data.unwrap();
    assert!(
        collection["heroes"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["owned"] == true))
    );
    assert!(
        collection["pets"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["owned"] == true))
    );
}

fn client_packet(timestamp: u32, proto_id: Id, message: impl Message) -> Vec<u8> {
    let mut payload = Vec::new();
    message.encode(&mut payload).unwrap();
    ClientPacket {
        timestamp,
        ack: 0,
        proto_id: proto_id as u16,
        payload,
    }
    .encode()
    .unwrap()
}
