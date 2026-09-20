CREATE TABLE player_ports (
    uid INTEGER NOT NULL,
    id INTEGER NOT NULL,
    pass_cnt INTEGER NOT NULL,
    is_c INTEGER NOT NULL,
    is_f INTEGER NOT NULL,
    port_id INTEGER NOT NULL,
    s1 INTEGER NOT NULL,
    s2 INTEGER NOT NULL,
    s3 INTEGER NOT NULL,
    t_cnt INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    all_cnt INTEGER NOT NULL,
    PRIMARY KEY (uid, id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_role_progress (
    uid INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    level INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    PRIMARY KEY (uid, role_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
