CREATE TABLE player_boss_rush_ranking_rewards (
    uid INTEGER NOT NULL REFERENCES players(uid) ON DELETE CASCADE,
    bid INTEGER NOT NULL,
    PRIMARY KEY (uid, bid)
);
