CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#c3a7f2',
    is_system INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS friend_tags (
    friend_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (friend_id, tag_id),
    FOREIGN KEY (friend_id) REFERENCES friends(user_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('默认', '#9b95a8', 1);
INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('重要', '#f2a7c3', 1);
INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('不赞', '#f28b8b', 1);
