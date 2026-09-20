CREATE TABLE player_rouge_run (
    uid INTEGER PRIMARY KEY,
    data BLOB NOT NULL,
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
