CREATE TABLE player_rouge_progress (
    uid INTEGER NOT NULL,
    rouge_id INTEGER NOT NULL,
    pass_count INTEGER NOT NULL,
    PRIMARY KEY (uid, rouge_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
