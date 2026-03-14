-- config 表：键值对存储应用配置
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- daily_state 表：每日点赞状态追踪
CREATE TABLE IF NOT EXISTS daily_state (
    date TEXT PRIMARY KEY NOT NULL,
    liked_count INTEGER DEFAULT 0,
    target_count INTEGER DEFAULT 50,
    is_completed INTEGER DEFAULT 0,
    last_run_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 默认配置值
INSERT OR IGNORE INTO config (key, value) VALUES
    ('daily_limit', '50'),
    ('times_per_friend', '10'),
    ('schedule_hour', '0'),
    ('schedule_minute', '5'),
    ('auto_start', 'false'),
    ('reply_like_enabled', 'false'),
    ('napcat_path', ''),
    ('qq_number', ''),
    ('qq_nickname', '');
