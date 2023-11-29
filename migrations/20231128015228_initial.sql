
CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL
);

CREATE TABLE user_session (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT REFERENCES user(id) NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id)
);
