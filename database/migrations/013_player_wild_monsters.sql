CREATE TABLE player_monster_points (
    uid INTEGER NOT NULL,
    day INTEGER NOT NULL,
    point_id TEXT NOT NULL,
    PRIMARY KEY (uid, point_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_region_coin_daily (
    uid INTEGER NOT NULL,
    day INTEGER NOT NULL,
    coin_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY (uid, coin_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
