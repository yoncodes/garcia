CREATE TABLE player_tasks (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    status INTEGER NOT NULL,
    picked_at INTEGER NOT NULL,
    progress INTEGER NOT NULL,
    total INTEGER NOT NULL,
    PRIMARY KEY (uid, id)
);
