CREATE TABLE IF NOT EXISTS player_achievements (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    took_at INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
