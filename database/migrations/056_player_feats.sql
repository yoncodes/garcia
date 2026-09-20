CREATE TABLE player_feats (
    uid INTEGER NOT NULL,
    feat_id INTEGER NOT NULL,
    status INTEGER NOT NULL,
    PRIMARY KEY (uid, feat_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
