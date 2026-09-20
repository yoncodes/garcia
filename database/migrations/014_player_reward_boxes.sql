CREATE TABLE player_reward_boxes (
    uid INTEGER NOT NULL,
    id TEXT NOT NULL,
    status INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
