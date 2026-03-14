-- 好友信息缓存表（每次批量点赞时同步更新）
CREATE TABLE IF NOT EXISTS friends (
    user_id INTEGER PRIMARY KEY,           -- QQ 号
    nickname TEXT NOT NULL DEFAULT '',      -- 昵称
    remark TEXT NOT NULL DEFAULT '',        -- 备注名
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- api_port 默认配置
INSERT OR IGNORE INTO config (key, value) VALUES ('api_port', '3000');
