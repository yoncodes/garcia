ALTER TABLE player_role_progress
ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

ALTER TABLE player_role_progress
ADD COLUMN maid_qua INTEGER NOT NULL DEFAULT 0;

CREATE TABLE player_role_rank_awards (
    uid INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    rank INTEGER NOT NULL,
    PRIMARY KEY (uid, role_id, rank),
    FOREIGN KEY (uid, role_id) REFERENCES player_role_progress(uid, role_id)
        ON DELETE CASCADE
);
