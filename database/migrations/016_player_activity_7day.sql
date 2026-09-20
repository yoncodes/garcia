CREATE TABLE IF NOT EXISTS player_activity_7day_claims (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    took_at INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS player_activity_7day_point_claims (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
