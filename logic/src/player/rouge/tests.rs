use super::*;

#[test]
fn rouge_list_uses_level_and_previous_clear_requirements() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    assert!(player.available_rouges(&tables).is_empty());

    player.level = 35;
    let first = player.available_rouges(&tables);
    assert_eq!(
        first.iter().map(|row| row.rouge_id).collect::<Vec<_>>(),
        vec![1, 5, 9]
    );

    player.rouge_progress.insert(1, 1);
    let unlocked = player.available_rouges(&tables);
    assert_eq!(
        unlocked
            .iter()
            .map(|row| (row.rouge_id, row.pass))
            .collect::<Vec<_>>(),
        vec![(1, 1), (2, 0), (5, 0), (9, 0)]
    );
}

#[test]
fn rouge_technology_uses_table_cost_and_any_predecessor() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.items.push(DcNetDataItem {
        user_item_id: 1,
        item_id: ROUGE_TECH_CURRENCY_ID,
        amount: 100,
        ..Default::default()
    });

    assert_eq!(
        player.unlock_rouge_technology(5, &tables),
        Err(RougeTechnologyError::Locked(5))
    );
    assert_eq!(
        player.unlock_rouge_technology(1, &tables).unwrap().amount,
        70
    );
    assert_eq!(
        player.unlock_rouge_technology(2, &tables).unwrap().amount,
        40
    );
    assert_eq!(
        player.unlock_rouge_technology(5, &tables).unwrap().amount,
        10
    );
    assert_eq!(player.rouge_technology, vec![1, 2, 5]);
}

#[test]
fn rouge_score_claim_takes_every_reached_table_reward_once() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_score = 1_500;

    let (reward, point_at) = player.claim_rouge_score_rewards(&tables, 1_000).unwrap();
    assert_eq!(point_at, 3);
    assert_eq!(player.rouge_score_point_at, 3);
    assert!(
        reward
            .items
            .iter()
            .any(|item| item.item_id == 100_100_003 && item.amount >= 50_000)
    );
    assert_eq!(
        player.claim_rouge_score_rewards(&tables, 1_001),
        Err(RougeScoreRewardError::NothingToClaim)
    );
}

#[test]
fn rouge_start_uses_tables_and_persists_team_and_status() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.level = 25;
    player.rouge_technology.push(1);
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;
    let role = DcNetDataRougeRole {
        game_role_id: role_id,
        hp: 1_000,
        ..Default::default()
    };

    let run = player
        .start_rouge(1, vec![role], Vec::new(), 1, &tables)
        .unwrap();
    let info = run.info.as_ref().unwrap();
    assert_eq!((info.rouge_id, info.coin, info.first_buff_tag), (1, 400, 1));
    assert_eq!(
        run.evt.as_ref().unwrap().event_id,
        RougeEvent::ReBuff as i32
    );
    assert_eq!(run.next.as_ref().unwrap().node_id, 1010);
    assert_eq!(
        player.start_rouge(1, vec![role], Vec::new(), 1, &tables),
        Err(RougeRunError::AlreadyActive)
    );

    let rerolled = player.regen_rouge_buff(&tables).unwrap();
    assert_eq!(rerolled.info.as_ref().unwrap().coin, 350);
    let offered = rerolled.evt.as_ref().unwrap().buffs[0];
    let selected = player.select_rouge_buff(offered).unwrap();
    assert_eq!(
        selected.evt.as_ref().unwrap().status,
        RougeEventStatus::ResFinished as i32
    );
    assert!(selected.info.as_ref().unwrap().buffs.contains(&offered));

    let entered = player.enter_rouge(1, &tables).unwrap();
    let entered_info = entered.info.as_ref().unwrap();
    assert_eq!(
        (entered_info.node_group_id, entered_info.node_id),
        (110, 1010)
    );
    assert!(entered_info.game_play_id > 0);
    assert_eq!(entered.next.as_ref().unwrap().node_id, 1020);
    let reported = player.report_rouge_node(vec![role], &tables).unwrap();
    assert_eq!(reported.info.as_ref().unwrap().total_port_pass, 1);
    assert_eq!(
        reported.evt.as_ref().unwrap().status,
        RougeEventStatus::ResBattleFinished as i32
    );
    let subevent_item = reported.subevt.as_ref().unwrap().subevt_item_id;
    assert!(subevent_item > 0);
    let collected = player.claim_rogue_subevent_item(&tables).unwrap();
    assert!(
        collected
            .info
            .as_ref()
            .unwrap()
            .items
            .contains(&subevent_item)
    );
    assert!(collected.subevt.as_ref().unwrap().item_finished);
    let battle_buff = reported.evt.as_ref().unwrap().buffs[0];
    assert_eq!(
        player
            .select_rouge_buff(battle_buff)
            .unwrap()
            .evt
            .unwrap()
            .status,
        RougeEventStatus::ResFinished as i32
    );

    player.set_rouge_status(1).unwrap();
    let changed = DcNetDataRougeRole { hp: 500, ..role };
    player.save_rouge_roles(vec![changed]).unwrap();
    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    let restored_info = restored.rouge_run.unwrap().info.unwrap();
    assert_eq!(restored_info.status, 1);
    assert_eq!(restored_info.roles, vec![changed]);
}

#[test]
fn rouge_coinbox_and_rest_apply_table_effects_once() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            coin: 100,
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReCoinbox as i32,
            coin: tables.rogue_rules.coinbox_default,
            ..Default::default()
        }),
        ..Default::default()
    });

    let opened = player.open_rouge_coinbox(&tables).unwrap();
    assert_eq!(opened.info.as_ref().unwrap().coin, 600);
    assert_eq!(
        opened.evt.as_ref().unwrap().status,
        RougeEventStatus::ResFinished as i32
    );
    assert_eq!(
        player.open_rouge_coinbox(&tables),
        Err(RougeRunError::InvalidEvent)
    );

    let role = DcNetDataRougeRole {
        game_role_id: 7,
        hp: 100,
        ..Default::default()
    };
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            pressure: 50,
            roles: vec![role],
            init_roles: [(7, 1_000)].into_iter().collect(),
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReRest as i32,
            ..Default::default()
        }),
        ..Default::default()
    });

    let rested = player.rest_during_rogue_run(&tables).unwrap();
    let info = rested.info.as_ref().unwrap();
    assert_eq!(info.pressure, 30);
    assert_eq!(info.roles[0].hp, 700);
    assert_eq!(
        rested.evt.as_ref().unwrap().status,
        RougeEventStatus::ResFinished as i32
    );
}

#[test]
fn rouge_blessing_applies_the_table_entry_and_records_its_curse() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            node_id: 1010,
            pressure: 50,
            roles: vec![DcNetDataRougeRole {
                game_role_id: 7,
                hp: 1_000,
                ..Default::default()
            }],
            init_roles: [(7, 1_000)].into_iter().collect(),
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReBlessing as i32,
            bless_id: 10,
            ..Default::default()
        }),
        ..Default::default()
    });

    let (opened, curse_index) = player.open_rouge_blessing(&tables).unwrap();
    let info = opened.info.as_ref().unwrap();
    assert_eq!(info.pressure, 20);
    assert_eq!(info.bless_list, vec![10]);
    assert!((0..5).contains(&curse_index));
    assert_eq!(
        opened.evt.as_ref().unwrap().status,
        RougeEventStatus::ResFinished as i32
    );
    assert_eq!(
        player.open_rouge_blessing(&tables),
        Err(RougeRunError::InvalidEvent)
    );
}

#[test]
fn rouge_finish_uses_stage_rewards_and_clears_the_persisted_run() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            coin: 1_000,
            total_port_pass: 6,
            total_boss_pass: 3,
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            status: RougeEventStatus::ResFinished as i32,
            ..Default::default()
        }),
        ..Default::default()
    });

    let outcome = player.finish_rouge(&tables, 1_000).unwrap();
    assert_eq!(outcome.rouge_score, 1_000);
    assert_eq!(outcome.rouge_info.total_boss_pass, 3);
    assert_eq!(outcome.items[0].item_id, ROUGE_TECH_CURRENCY_ID);
    assert_eq!(outcome.items[0].amount, 250);
    assert!(
        outcome
            .reward
            .items
            .iter()
            .any(|item| item.item_id == 100_100_002 && item.amount == 120)
    );
    assert_eq!(player.rouge_progress.get(&1), Some(&1));
    assert!(player.rouge_run.is_none());
}

#[test]
fn rouge_trade_tracks_stock_discount_items_and_auto_use_effects() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            coin: 1_000,
            pressure: 50,
            eff_list: vec![DcNetDataRougeEffect {
                id: 1,
                eff_id: RougeEffect::EffTradeDiscount as i32,
                eff_params: "-2000".into(),
                eternal: true,
                ..Default::default()
            }],
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReTrade as i32,
            status: RougeEventStatus::ResFinished as i32,
            regular_goods: [(1001, 2), (1007, 1)].into_iter().collect(),
            ..Default::default()
        }),
        ..Default::default()
    });

    let bought = player
        .buy_rouge_trade(1, &[1001, 1001, 1007], &tables)
        .unwrap();
    let info = bought.info.as_ref().unwrap();
    assert_eq!((info.coin, info.total_coin_spend), (440, 560));
    assert_eq!(info.items, vec![1001, 1001]);
    assert_eq!(info.buffs.len(), 1);
    assert!(bought.evt.as_ref().unwrap().regular_goods.is_empty());
    assert_eq!(
        player.buy_rouge_trade(1, &[1001], &tables),
        Err(RougeRunError::InsufficientRougeStock(1001))
    );

    let used = player.use_rouge_item(1001, &tables).unwrap();
    let info = used.info.as_ref().unwrap();
    assert_eq!(info.pressure, 40);
    assert_eq!(info.items, vec![1001]);
}

#[test]
fn rouge_npc_options_enforce_costs_and_apply_the_selected_table_effect() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let info = DcNetDataRougeInfo {
        rouge_id: 1,
        coin: 300,
        pressure: 10,
        roles: vec![DcNetDataRougeRole {
            game_role_id: 7,
            hp: 500,
            ..Default::default()
        }],
        init_roles: [(7, 1_000)].into_iter().collect(),
        ..Default::default()
    };
    assert!(is_rouge_npc_effect_enabled(13, &info, &tables));
    assert!(!is_rouge_npc_effect_enabled(14, &info, &tables));
    assert!(is_rouge_npc_effect_enabled(16, &info, &tables));
    assert!(!is_rouge_npc_effect_enabled(17, &info, &tables));

    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(info),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReNpc as i32,
            npc_id: 2,
            npc_effects: vec![DcNetDataNpcEffect {
                eff_id: 4,
                enabled: true,
            }],
            ..Default::default()
        }),
        ..Default::default()
    });

    let selected = player.select_rouge_npc_effect(4, &tables).unwrap();
    let info = selected.info.as_ref().unwrap();
    assert_eq!(info.pressure, 20);
    assert_eq!(info.buffs.len(), 1);
    assert_eq!(
        selected.evt.as_ref().unwrap().status,
        RougeEventStatus::ResFinished as i32
    );
}

#[test]
fn rouge_battle_applies_accumulated_effects_and_expires_timed_ones() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;
    let effect = |id, kind, value: &str, eternal, count| DcNetDataRougeEffect {
        id,
        eff_id: kind as i32,
        eff_params: value.into(),
        remain_cnt: count,
        eternal,
        total_cnt: count,
    };
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            node_id: 1010,
            game_play_id: 180_500_001,
            roles: vec![DcNetDataRougeRole {
                game_role_id: role_id,
                hp: 500,
                ..Default::default()
            }],
            init_roles: [(role_id, 1_000)].into_iter().collect(),
            eff_list: vec![
                effect(1, RougeEffect::EffCoinGain, "1000", true, 0),
                effect(2, RougeEffect::EffBattleCoinGain, "2000|1", false, 1),
                effect(3, RougeEffect::EffAllPressureGain, "1000", true, 0),
                effect(4, RougeEffect::EffBattlePressureGain, "2000|1", false, 1),
                effect(5, RougeEffect::EffRougeBuffCount, "2", true, 0),
                effect(6, RougeEffect::EffBattleHpRecover, "1000", true, 0),
            ],
            ..Default::default()
        }),
        evt: Some(DcNetDataRougeEvent {
            event_id: RougeEvent::ReBuff as i32,
            ..Default::default()
        }),
        ..Default::default()
    });

    let reported = player
        .report_rouge_node(
            vec![DcNetDataRougeRole {
                game_role_id: role_id,
                hp: 500,
                ..Default::default()
            }],
            &tables,
        )
        .unwrap();
    let info = reported.info.as_ref().unwrap();
    assert_eq!((info.coin, info.pressure, info.roles[0].hp), (130, 13, 600));
    assert_eq!(reported.evt.as_ref().unwrap().buff_times, 3);
    assert!(info.eff_list.iter().all(|effect| effect.eternal));
}

#[test]
fn unsuccessful_rouge_report_ends_run_and_preserves_result_snapshot() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let role_id = player.roles[0]
        .role_basic_info
        .as_ref()
        .unwrap()
        .game_role_id;
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            coin: 275,
            total_port_pass: 4,
            roles: vec![DcNetDataRougeRole {
                game_role_id: role_id,
                hp: 1_000,
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    });

    let snapshot = player
        .end_rouge_run_unsuccessfully(vec![DcNetDataRougeRole {
            game_role_id: role_id,
            hp: 0,
            ..Default::default()
        }])
        .unwrap();

    assert_eq!((snapshot.rouge_id, snapshot.coin), (1, 275));
    assert_eq!(snapshot.total_port_pass, 4);
    assert_eq!(snapshot.roles[0].hp, 0);
    assert!(player.rouge_run.is_none());
}

#[test]
fn abandoning_rouge_accepts_the_clients_empty_role_list() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            ..Default::default()
        }),
        ..Default::default()
    });

    assert_eq!(
        player
            .end_rouge_run_unsuccessfully(Vec::new())
            .unwrap()
            .rouge_id,
        1
    );
    assert!(player.rouge_run.is_none());
}

#[test]
fn rouge_boss_box_rejects_unresolved_server_reward_without_charging() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    // No active run.
    assert_eq!(
        player.open_rouge_boss_box(&tables, 0),
        Err(RougeRunError::NotActive)
    );

    // A run whose current port (180500010) is a boss port resolves its drop
    // (gameplay_port_rouge.bossbox_reward = 130400021).
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            game_play_id: 180500010,
            ..Default::default()
        }),
        subevt: Some(DcNetDataRougeSubEvt::default()),
        ..Default::default()
    });
    player.add_item(100100021, 1, &tables).unwrap();
    let items_before = player.items.clone();
    assert_eq!(
        player.open_rouge_boss_box(&tables, 0),
        Err(RougeRunError::InvalidConfig(130400021))
    );
    assert_eq!(player.items, items_before);
    assert!(
        !player
            .rouge_run
            .as_ref()
            .unwrap()
            .subevt
            .as_ref()
            .unwrap()
            .boss_box_opened
    );

    // A run with no boss port is rejected.
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            game_play_id: 0,
            ..Default::default()
        }),
        ..Default::default()
    });
    assert_eq!(
        player.open_rouge_boss_box(&tables, 0),
        Err(RougeRunError::InvalidEvent)
    );
}

#[test]
fn rouge_boss_box_uses_the_configured_energy_item() {
    let config = common::load_config().unwrap();
    let mut tables = GameTables::load(&config.paths.game_tables).unwrap();
    let energy_item_id = tables.cultivation_constants.diamond_item_id;
    tables.cultivation_constants.heat_item_id = energy_item_id;
    let reward_id = tables.rewards.rows[0].id;
    tables
        .rogue_ports
        .rows
        .iter_mut()
        .find(|port| port.id == 180_500_010)
        .unwrap()
        .boss_box_reward = reward_id;

    let mut player = Player::new(42, &tables, &config.account_defaults);
    player
        .add_item(energy_item_id, tables.rogue_rules.bossbox_phy_cost, &tables)
        .unwrap();
    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            game_play_id: 180_500_010,
            ..Default::default()
        }),
        subevt: Some(DcNetDataRougeSubEvt::default()),
        ..Default::default()
    });

    let (_, remain) = player.open_rouge_boss_box(&tables, 0).unwrap();

    assert_eq!((remain.item_id, remain.amount), (energy_item_id, 0));
    assert!(
        player
            .rouge_run
            .as_ref()
            .unwrap()
            .subevt
            .as_ref()
            .unwrap()
            .boss_box_opened
    );
}

#[test]
fn rouge_subevent_store_buy_consumes_stock_and_coin() {
    let config = common::load_config().unwrap();
    let tables = GameTables::load(&config.paths.game_tables).unwrap();
    let mut player = Player::new(42, &tables, &config.account_defaults);

    player.rouge_run = Some(DcNetDataRouge {
        info: Some(DcNetDataRougeInfo {
            rouge_id: 1,
            coin: 1_000,
            ..Default::default()
        }),
        subevt: Some(DcNetDataRougeSubEvt {
            regular_goods: [(1001, 2)].into_iter().collect(),
            ..Default::default()
        }),
        ..Default::default()
    });

    let bought = player
        .buy_rouge_subevent_store(1, &[1001, 1001], &tables)
        .unwrap();
    let info = bought.info.as_ref().unwrap();
    assert!(info.coin < 1_000);
    assert_eq!(info.items, vec![1001, 1001]);
    assert!(bought.subevt.as_ref().unwrap().regular_goods.is_empty());
    assert_eq!(
        player.buy_rouge_subevent_store(1, &[1001], &tables),
        Err(RougeRunError::InsufficientRougeStock(1001))
    );
}
