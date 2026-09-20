CREATE TABLE IF NOT EXISTS player_checkins (
    uid INTEGER NOT NULL,
    aid INTEGER NOT NULL,
    check_days INTEGER NOT NULL,
    last_check_day INTEGER NOT NULL,
    PRIMARY KEY (uid, aid),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS player_checkin_claims (
    uid INTEGER NOT NULL,
    aid INTEGER NOT NULL,
    get_day INTEGER NOT NULL,
    PRIMARY KEY (uid, aid, get_day),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
