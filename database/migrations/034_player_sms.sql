CREATE TABLE player_sms (
    uid INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    read INTEGER NOT NULL,
    selected TEXT NOT NULL,
    PRIMARY KEY (uid, group_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
