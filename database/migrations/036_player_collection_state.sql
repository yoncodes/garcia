CREATE TABLE player_collection_suit_rewards (
    uid INTEGER NOT NULL,
    suit_id INTEGER NOT NULL,
    step INTEGER NOT NULL,
    PRIMARY KEY (uid, suit_id, step),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_collection_places (
    uid INTEGER NOT NULL,
    platform_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL,
    PRIMARY KEY (uid, platform_id),
    UNIQUE (uid, collection_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
