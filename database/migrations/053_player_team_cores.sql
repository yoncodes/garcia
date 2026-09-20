CREATE TABLE player_team_cores (
    uid INTEGER NOT NULL,
    instance_id INTEGER NOT NULL,
    pos INTEGER NOT NULL,
    locked INTEGER NOT NULL,
    equipped_id INTEGER NOT NULL,
    core_id INTEGER NOT NULL,
    PRIMARY KEY (uid, instance_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
