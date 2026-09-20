CREATE TABLE player_heat_exchange (
    uid INTEGER PRIMARY KEY REFERENCES players(uid) ON DELETE CASCADE,
    day INTEGER NOT NULL,
    exchange_count INTEGER NOT NULL
);
