CREATE TABLE player_dungeons (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
