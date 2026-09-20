CREATE TABLE player_daily_state (
    uid INTEGER PRIMARY KEY,
    day INTEGER NOT NULL,
    activity INTEGER NOT NULL,
    reward_progress INTEGER NOT NULL,
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_daily_tasks (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    taken INTEGER NOT NULL,
    progress INTEGER NOT NULL,
    total INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
