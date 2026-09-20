CREATE TABLE player_items (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    user_item_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    remain_sec INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    PRIMARY KEY (uid, item_id)
);
