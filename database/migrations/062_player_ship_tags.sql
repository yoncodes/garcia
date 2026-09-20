CREATE TABLE player_ship_tags (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (uid, tag_id)
);
