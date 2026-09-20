CREATE TABLE player_activity_boss (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    aid INTEGER NOT NULL,
    day INTEGER NOT NULL,
    damage INTEGER NOT NULL,
    daily_damage INTEGER NOT NULL
);

CREATE TABLE player_activity_boss_roles (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (uid, position)
);

CREATE TABLE player_activity_boss_rewards (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    reward_type INTEGER NOT NULL,
    reward_id INTEGER NOT NULL,
    state INTEGER NOT NULL,
    PRIMARY KEY (uid, reward_type, reward_id)
);
