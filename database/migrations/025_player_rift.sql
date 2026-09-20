CREATE TABLE player_rift (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    rid INTEGER NOT NULL,
    current_port_id INTEGER NOT NULL,
    best_buff_score INTEGER NOT NULL,
    best_score INTEGER NOT NULL,
    best_stage INTEGER NOT NULL,
    completion_count INTEGER NOT NULL,
    run_buff_score INTEGER NOT NULL,
    run_score INTEGER NOT NULL,
    run_duration INTEGER NOT NULL,
    active INTEGER NOT NULL
);

CREATE TABLE player_rift_roles (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (uid, position)
);

CREATE TABLE player_rift_tasks (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    task_id INTEGER NOT NULL,
    PRIMARY KEY (uid, task_id)
);
