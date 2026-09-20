CREATE TABLE player_gold_coins (
    uid INTEGER NOT NULL,
    city_id INTEGER NOT NULL,
    coin_id TEXT NOT NULL,
    PRIMARY KEY (uid, coin_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
