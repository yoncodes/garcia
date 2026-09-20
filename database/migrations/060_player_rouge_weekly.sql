CREATE TABLE player_rouge_weekly (
    uid INTEGER PRIMARY KEY,
    score INTEGER NOT NULL,
    score_point_at INTEGER NOT NULL,
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
