CREATE TABLE IF NOT EXISTS player_mails (
    uid INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    is_read INTEGER NOT NULL,
    taken INTEGER NOT NULL,
    sent_at INTEGER NOT NULL,
    sender INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    sys_mail_id INTEGER NOT NULL,
    parameter TEXT NOT NULL,
    PRIMARY KEY (uid, email_id),
    FOREIGN KEY (uid) REFERENCES players(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS player_mail_gifts (
    uid INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    reward INTEGER NOT NULL,
    reward_type INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY (uid, email_id, position),
    FOREIGN KEY (uid, email_id) REFERENCES player_mails(uid, email_id) ON DELETE CASCADE
);
