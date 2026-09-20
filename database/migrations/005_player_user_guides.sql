CREATE TABLE player_user_guides (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    gid TEXT NOT NULL,
    PRIMARY KEY (uid, gid)
);
