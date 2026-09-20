CREATE TABLE player_equipment_groups (
    uid INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    game_role_id INTEGER NOT NULL,
    group_name TEXT NOT NULL,
    PRIMARY KEY (uid, group_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_equipment_group_items (
    uid INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    user_equip_id INTEGER NOT NULL,
    PRIMARY KEY (uid, group_id, position),
    FOREIGN KEY (uid, group_id) REFERENCES player_equipment_groups(uid, group_id) ON DELETE CASCADE
);
