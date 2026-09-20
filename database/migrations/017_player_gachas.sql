CREATE TABLE IF NOT EXISTS player_gachas (
    uid INTEGER NOT NULL,
    gacha_id INTEGER NOT NULL,
    all_count INTEGER NOT NULL,
    gacha_count_10 INTEGER NOT NULL,
    reward_cnt INTEGER NOT NULL,
    reward_num INTEGER NOT NULL,
    taken_new_reward INTEGER NOT NULL,
    had_take_reward INTEGER NOT NULL,
    PRIMARY KEY (uid, gacha_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
