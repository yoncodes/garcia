CREATE TABLE player_profile_unlocks (
    uid INTEGER NOT NULL,
    profile_type INTEGER NOT NULL,
    profile_id INTEGER NOT NULL,
    PRIMARY KEY (uid, profile_type, profile_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
