use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn sms_unlocks_from_task_state_and_read_choices_persist() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.extend(
        [1_501_005_020, 1_501_005_030, 1_501_005_040]
            .into_iter()
            .map(|id| DcNetDataTaskStatus {
                id,
                status: TaskStatus::Done as i32,
                ..Default::default()
            }),
    );

    assert!(!player.sync_sms(&tables).is_empty());
    assert!(player.sms.iter().any(|message| message.id == 410_101));
    let (message, changed) = player.read_sms(410_101, vec![2], &tables).unwrap();
    assert!(changed);
    assert!(message.read);
    assert_eq!(message.selected, [2]);

    let restored = Player::from_record(player.to_record(), &tables, &config.account_defaults);
    assert_eq!(restored.sms, player.sms);
}

#[test]
fn picked_sms_remains_after_the_unlock_task_completes() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    player.tasks.push(DcNetDataTaskStatus {
        id: 1_501_022_010,
        status: TaskStatus::Picked as i32,
        ..Default::default()
    });
    assert!(!player.sync_sms(&tables).is_empty());
    assert!(player.sms.iter().any(|message| message.id == 420_108));

    player
        .tasks
        .iter_mut()
        .find(|task| task.id == 1_501_022_010)
        .unwrap()
        .status = TaskStatus::Done as i32;
    assert!(player.sync_sms(&tables).is_empty());
    assert!(player.sms.iter().any(|message| message.id == 420_108));
    assert_eq!(
        player.read_sms(420_108, vec![5], &tables).unwrap_err(),
        SmsError::InvalidSelection(5)
    );
}
