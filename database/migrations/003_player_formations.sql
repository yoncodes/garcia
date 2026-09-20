CREATE TABLE player_formations (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    formation_id INTEGER NOT NULL,
    remark TEXT NOT NULL,
    PRIMARY KEY (uid, formation_id)
);

CREATE TABLE player_formation_positions (
    uid INTEGER NOT NULL,
    formation_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    game_role_id INTEGER NOT NULL,
    key_num INTEGER NOT NULL,
    PRIMARY KEY (uid, formation_id, position),
    FOREIGN KEY (uid, formation_id)
        REFERENCES player_formations(uid, formation_id) ON DELETE CASCADE
);
