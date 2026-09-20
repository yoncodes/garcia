CREATE TABLE player_region_progress (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    region_id INTEGER NOT NULL,
    level INTEGER NOT NULL,
    PRIMARY KEY (uid, region_id)
);
