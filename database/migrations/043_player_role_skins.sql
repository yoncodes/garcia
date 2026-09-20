CREATE TABLE player_skins (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    skin_id INTEGER NOT NULL,
    PRIMARY KEY (uid, skin_id)
);

ALTER TABLE player_role_progress
ADD COLUMN skin_id INTEGER NOT NULL DEFAULT 0;
