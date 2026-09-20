CREATE TABLE player_role_attrs (
    uid INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    mp INTEGER NOT NULL,
    ep INTEGER NOT NULL,
    hp INTEGER NOT NULL,
    PRIMARY KEY (uid, role_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
