CREATE TABLE player_boss_rush (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    bid INTEGER NOT NULL
);

CREATE TABLE player_boss_rush_ports (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    pid INTEGER NOT NULL,
    damage INTEGER NOT NULL,
    PRIMARY KEY (uid, pid)
);

CREATE TABLE player_boss_rush_port_roles (
    uid INTEGER NOT NULL,
    pid INTEGER NOT NULL,
    position INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (uid, pid, position),
    FOREIGN KEY (uid, pid) REFERENCES player_boss_rush_ports(uid, pid) ON DELETE CASCADE
);

CREATE TABLE player_boss_rush_rewards (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    sid INTEGER NOT NULL,
    PRIMARY KEY (uid, sid)
);
