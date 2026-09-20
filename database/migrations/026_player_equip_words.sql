CREATE TABLE player_equip_deputy_words (
    uid INTEGER NOT NULL,
    user_equip_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    word_id INTEGER NOT NULL,
    PRIMARY KEY (uid, user_equip_id, position),
    FOREIGN KEY (uid, user_equip_id) REFERENCES player_equips(uid, user_equip_id)
        ON DELETE CASCADE
);

CREATE TABLE player_equip_random_words (
    uid INTEGER NOT NULL,
    user_equip_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    word_id INTEGER NOT NULL,
    PRIMARY KEY (uid, user_equip_id, position),
    FOREIGN KEY (uid, user_equip_id) REFERENCES player_equips(uid, user_equip_id)
        ON DELETE CASCADE
);
