CREATE TABLE player_monster_manuals (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    gameplay_id INTEGER NOT NULL,
    enemy_hash INTEGER NOT NULL CHECK (enemy_hash BETWEEN 0 AND 4294967295),
    PRIMARY KEY (uid, gameplay_id, enemy_hash)
);
