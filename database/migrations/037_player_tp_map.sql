CREATE TABLE player_tp_map (
    uid INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (uid, key),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
