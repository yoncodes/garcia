CREATE TABLE player_gacha_logs (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    log_id INTEGER NOT NULL,
    gacha_id INTEGER NOT NULL,
    reward INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (uid, log_id)
);
