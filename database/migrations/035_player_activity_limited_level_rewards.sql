CREATE TABLE player_activity_limited_level_rewards (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);
