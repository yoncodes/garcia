use super::*;

fn fixture() -> (GameTables, common::config::ServerConfig) {
    let versions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tables");
    (
        GameTables::load(&versions).unwrap(),
        common::load_config().unwrap(),
    )
}

#[test]
fn trials_unlock_from_their_table_conditions() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let trial_id = 140_200_000;
    let required_task = 150_100_004;

    assert!(
        !player
            .trials(&tables, 123)
            .iter()
            .any(|trial| trial.trial_id == trial_id)
    );
    player.tasks.push(DcNetDataTaskStatus {
        id: required_task,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    assert!(
        player
            .trials(&tables, 123)
            .iter()
            .any(|trial| trial.trial_id == trial_id)
    );
}

#[test]
fn trial_lifecycle_uses_configured_team_rewards_task_and_return_point() {
    let (tables, config) = fixture();
    let mut player = Player::new(42, &tables, &config.account_defaults);
    let trial_id = 140_200_000;
    let trial = tables.trials.get(trial_id).unwrap().clone();
    player.tasks.push(DcNetDataTaskStatus {
        id: 150_100_004,
        status: TaskStatus::Done as i32,
        ..Default::default()
    });
    let expected_roles = player
        .formations
        .iter()
        .find(|formation| formation.formation_id == player.cur_form)
        .unwrap()
        .poss
        .iter()
        .map(|position| position.game_role_id)
        .collect::<Vec<_>>();
    let before = trial
        .reward
        .iter()
        .map(|entry| {
            (
                entry.key,
                player
                    .items
                    .iter()
                    .find(|item| item.item_id == entry.key)
                    .map_or(0, |item| item.amount),
            )
        })
        .collect::<HashMap<_, _>>();

    assert_eq!(
        player.save_trial(trial_id, &tables, 123).unwrap(),
        expected_roles
    );
    assert_eq!(
        player.trial_info(trial_id, &tables, 123).unwrap().trial_id,
        trial_id
    );
    let outcome = player.finish_trial(trial_id, &tables, 124).unwrap();

    assert_eq!(
        (outcome.trial_id, outcome.task.id),
        (trial_id, trial.finish_task_id)
    );
    assert_eq!(outcome.task.status, TaskStatus::Done as i32);
    assert_eq!(
        outcome.local,
        Some(DcNetDataLocal {
            region: trial_id,
            local: trial.complete_point.clone(),
            local2: String::new(),
        })
    );
    for entry in &trial.reward {
        assert_eq!(
            player
                .items
                .iter()
                .find(|item| item.item_id == entry.key)
                .unwrap()
                .amount,
            before[&entry.key] + entry.value
        );
    }
    assert!(matches!(
        player.finish_trial(trial_id, &tables, 125),
        Err(TrialError::AlreadyComplete(id)) if id == trial_id
    ));
}
