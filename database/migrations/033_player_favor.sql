CREATE TABLE player_favors (
    uid INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    level INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    PRIMARY KEY (uid, character_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_favor_daily (
    uid INTEGER PRIMARY KEY,
    day INTEGER NOT NULL,
    touches INTEGER NOT NULL,
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
