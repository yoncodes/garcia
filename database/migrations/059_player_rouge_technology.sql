CREATE TABLE player_rouge_technology (
    uid INTEGER NOT NULL,
    technology_id INTEGER NOT NULL,
    PRIMARY KEY (uid, technology_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
