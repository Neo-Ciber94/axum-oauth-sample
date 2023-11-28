
CREATE TABLE user (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL
);

CREATE TABLE user_session (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES user(id) NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id)
);
