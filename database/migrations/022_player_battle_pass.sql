CREATE TABLE player_battle_pass (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    paid_status INTEGER NOT NULL,
    level INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    exp_week INTEGER NOT NULL,
    week INTEGER NOT NULL
);

CREATE TABLE player_battle_pass_rewards (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    level INTEGER NOT NULL,
    state INTEGER NOT NULL,
    PRIMARY KEY (uid, level)
);

CREATE TABLE player_battle_pass_tasks (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);
