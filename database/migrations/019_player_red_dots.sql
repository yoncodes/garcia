CREATE TABLE IF NOT EXISTS player_checked_red_dots (
    uid INTEGER NOT NULL,
    id TEXT NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
