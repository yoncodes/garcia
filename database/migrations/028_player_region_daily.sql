CREATE TABLE player_region_daily (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    ticket_day INTEGER NOT NULL
);
