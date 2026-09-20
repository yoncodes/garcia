use super::*;

fn tables() -> GameTables {
    GameTables::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables"))
        .unwrap()
}

#[test]
fn builds_captured_rift_info_and_tasks_from_tables() {
    let tables = tables();
    let zone_offset = -25_200;
    let now = 1_787_522_879;

    let info = rift_info(&tables, now, zone_offset).unwrap();
    assert_eq!(info.id, 8);
    assert_eq!(info.battle_remain_sec, 46_320);
    assert_eq!(info.rest_remain_sec, 132_781);
    assert_eq!(info.battle_end_time, 1_787_569_199);
    assert_eq!(info.rest_end_time, 1_787_655_660);

    let tasks = rift_tasks(
        &tables,
        current_rift(&tables, now, zone_offset).unwrap(),
        None,
    );
    assert_eq!(tasks.len(), 24);
    assert_eq!((tasks[0].id, tasks[0].total), (6001, 5));
    assert_eq!((tasks[23].id, tasks[23].total), (6024, 140_000));
    assert_eq!(tasks[0].reward.get(&100_100_030), Some(&100));
    assert_eq!(tasks[8].reward.get(&101_200_024), Some(&1));
}

#[test]
fn maps_each_rift_task_condition_to_persisted_progress() {
    let tables = tables();
    let state = crate::logic::RiftState {
        best_stage: 8,
        best_buff_score: 22,
        completion_count: 4,
        best_score: 35_000,
        ..Default::default()
    };
    let tasks = rift_tasks(&tables, tables.rifts.get(8).unwrap(), Some(&state));

    assert_eq!(
        tasks.iter().find(|task| task.id == 6002).unwrap().progress,
        8
    );
    assert_eq!(
        tasks.iter().find(|task| task.id == 6012).unwrap().progress,
        22
    );
    assert_eq!(
        tasks.iter().find(|task| task.id == 6017).unwrap().progress,
        4
    );
    assert_eq!(
        tasks.iter().find(|task| task.id == 6021).unwrap().progress,
        35_000
    );
}

#[test]
fn keeps_current_rift_during_its_rest_window() {
    let tables = tables();
    let zone_offset = -25_200;
    let after_battle = 1_787_570_000;

    let info = rift_info(&tables, after_battle, zone_offset).unwrap();
    assert_eq!(info.id, 8);
    assert_eq!(info.battle_remain_sec, 0);
    assert!(info.rest_remain_sec > 0);
}
