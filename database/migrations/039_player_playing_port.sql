CREATE TABLE player_playing_port (
    uid INTEGER PRIMARY KEY,
    port_index_id INTEGER NOT NULL,
    port_info TEXT NOT NULL,
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);
