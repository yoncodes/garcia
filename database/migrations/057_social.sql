CREATE TABLE friendships (
    uid_low INTEGER NOT NULL,
    uid_high INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (uid_low, uid_high),
    CHECK (uid_low < uid_high),
    FOREIGN KEY (uid_low) REFERENCES players(uid) ON DELETE CASCADE,
    FOREIGN KEY (uid_high) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE friend_requests (
    requester_uid INTEGER NOT NULL,
    receiver_uid INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (requester_uid, receiver_uid),
    CHECK (requester_uid <> receiver_uid),
    FOREIGN KEY (requester_uid) REFERENCES players(uid) ON DELETE CASCADE,
    FOREIGN KEY (receiver_uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE INDEX friend_requests_receiver ON friend_requests(receiver_uid);

CREATE TABLE friend_gifts (
    sender_uid INTEGER NOT NULL,
    receiver_uid INTEGER NOT NULL,
    day INTEGER NOT NULL,
    claimed INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (sender_uid, receiver_uid, day),
    CHECK (sender_uid <> receiver_uid),
    CHECK (claimed IN (0, 1)),
    FOREIGN KEY (sender_uid) REFERENCES players(uid) ON DELETE CASCADE,
    FOREIGN KEY (receiver_uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE INDEX friend_gifts_receiver_day ON friend_gifts(receiver_uid, day);
