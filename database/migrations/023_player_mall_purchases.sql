CREATE TABLE player_mall_purchases (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    goods_id INTEGER NOT NULL,
    bought INTEGER NOT NULL,
    period INTEGER NOT NULL,
    PRIMARY KEY (uid, goods_id)
);
