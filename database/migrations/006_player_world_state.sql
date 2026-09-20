CREATE TABLE player_interact_objs (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    object_id TEXT NOT NULL,
    interactive INTEGER NOT NULL,
    status INTEGER NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (uid, object_id)
);

CREATE TABLE player_challenges (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id TEXT NOT NULL,
    stars INTEGER NOT NULL,
    finished INTEGER NOT NULL,
    claimed INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);

CREATE TABLE player_albums (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    status INTEGER NOT NULL,
    sort INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);

CREATE TABLE player_dense_fogs (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);
