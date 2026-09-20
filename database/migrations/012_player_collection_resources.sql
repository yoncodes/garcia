CREATE TABLE player_collection_resources (
    uid INTEGER NOT NULL,
    collection_id TEXT NOT NULL,
    remaining INTEGER NOT NULL,
    PRIMARY KEY (uid, collection_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
