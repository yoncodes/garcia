CREATE TABLE players (
    uid INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    nickname TEXT NOT NULL,
    level INTEGER NOT NULL,
    gameplay_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    last_login_time INTEGER NOT NULL,
    cur_form INTEGER NOT NULL,
    profile_avatar INTEGER NOT NULL,
    profile_card INTEGER NOT NULL,
    profile_title INTEGER NOT NULL,
    profile_frame INTEGER NOT NULL,
    region INTEGER NOT NULL
);

CREATE TABLE player_city_guides (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    id TEXT NOT NULL,
    PRIMARY KEY (uid, id)
);

CREATE TABLE player_locals (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    region INTEGER NOT NULL,
    local TEXT NOT NULL,
    local2 TEXT NOT NULL,
    PRIMARY KEY (uid, region)
);
