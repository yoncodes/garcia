CREATE TABLE player_item_spent (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    item_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY (uid, item_id)
);

CREATE TABLE player_dungeon_clears (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    dungeon_type INTEGER NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (uid, dungeon_type)
);
