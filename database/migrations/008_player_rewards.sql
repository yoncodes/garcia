CREATE TABLE player_partners (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    partner_id INTEGER NOT NULL,
    brek INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    locked INTEGER NOT NULL,
    lv INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    reson_lv INTEGER NOT NULL,
    skill_lv INTEGER NOT NULL,
    creat_at INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);

CREATE TABLE player_equips (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    user_equip_id INTEGER NOT NULL,
    equip_id INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    in_group INTEGER NOT NULL,
    locked INTEGER NOT NULL,
    level INTEGER NOT NULL,
    equiped_role INTEGER NOT NULL,
    pos INTEGER NOT NULL,
    main_words_id INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    minnum INTEGER NOT NULL,
    tmp_word INTEGER NOT NULL,
    tmp_word_idx INTEGER NOT NULL,
    creat_at INTEGER NOT NULL,
    PRIMARY KEY (uid, user_equip_id)
);

CREATE TABLE player_collections (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    cid INTEGER NOT NULL,
    in_time INTEGER NOT NULL,
    reward INTEGER NOT NULL,
    PRIMARY KEY (uid, cid)
);
