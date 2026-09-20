CREATE TABLE player_version_challenges (
    uid INTEGER NOT NULL,
    challenge_id INTEGER NOT NULL,
    star1 INTEGER NOT NULL,
    star2 INTEGER NOT NULL,
    star3 INTEGER NOT NULL,
    PRIMARY KEY (uid, challenge_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE player_version_challenge_rewards (
    uid INTEGER NOT NULL,
    reward_id INTEGER NOT NULL,
    PRIMARY KEY (uid, reward_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
