CREATE TABLE player_role_talents (
    uid INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    level INTEGER NOT NULL,
    PRIMARY KEY (uid, role_id, position),
    FOREIGN KEY (uid, role_id) REFERENCES player_role_progress(uid, role_id)
        ON DELETE CASCADE
);
