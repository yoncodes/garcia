CREATE TABLE player_missions (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    misson_id INTEGER NOT NULL,
    total_num INTEGER NOT NULL,
    curr_num INTEGER NOT NULL,
    taken INTEGER NOT NULL,
    take_time INTEGER NOT NULL,
    type INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (uid, misson_id)
);
