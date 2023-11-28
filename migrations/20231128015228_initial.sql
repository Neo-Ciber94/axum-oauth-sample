
CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL
);

CREATE TABLE user_session (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT REFERENCES user(id) NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id)
);
