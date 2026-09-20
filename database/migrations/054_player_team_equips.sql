CREATE TABLE player_team_equips (
    uid INTEGER NOT NULL,
    instance_id INTEGER NOT NULL,
    equipped_formation_id INTEGER NOT NULL,
    locked INTEGER NOT NULL,
    team_equip_id INTEGER NOT NULL,
    level INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    main_words_id INTEGER NOT NULL,
    pos_num INTEGER NOT NULL,
    PRIMARY KEY (uid, instance_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
