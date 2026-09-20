CREATE TABLE player_activity_challenge_points (
    uid INTEGER NOT NULL,
    reward_id INTEGER NOT NULL,
    PRIMARY KEY (uid, reward_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
