CREATE TABLE player_skillstones (
    uid INTEGER NOT NULL,
    user_stone_id INTEGER NOT NULL,
    stone_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    locked INTEGER NOT NULL,
    equipped_role INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    PRIMARY KEY (uid, user_stone_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
