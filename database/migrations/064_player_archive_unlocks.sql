CREATE TABLE player_archive_unlocks (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    archive_id INTEGER NOT NULL,
    unlocked_at INTEGER NOT NULL,
    PRIMARY KEY (uid, archive_id)
);
