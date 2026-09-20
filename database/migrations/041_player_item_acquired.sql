CREATE TABLE player_item_acquired (
    uid INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY (uid, item_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
