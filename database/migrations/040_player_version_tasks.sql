CREATE TABLE player_version_task_claims (
    uid INTEGER NOT NULL,
    task_id INTEGER NOT NULL,
    PRIMARY KEY (uid, task_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
