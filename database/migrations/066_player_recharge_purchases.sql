CREATE TABLE player_recharge_purchases (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    recharge_id INTEGER NOT NULL,
    purchase_count INTEGER NOT NULL,
    last_bought_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    PRIMARY KEY (uid, recharge_id)
);
