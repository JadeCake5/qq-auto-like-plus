-- 为 daily_state 表添加分类计数列
ALTER TABLE daily_state ADD COLUMN scheduled_count INTEGER DEFAULT 0;
ALTER TABLE daily_state ADD COLUMN reply_count INTEGER DEFAULT 0;
ALTER TABLE daily_state ADD COLUMN manual_count INTEGER DEFAULT 0;

-- 点赞历史记录表
CREATE TABLE IF NOT EXISTS like_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    times INTEGER NOT NULL DEFAULT 10,
    like_type TEXT NOT NULL CHECK(like_type IN ('scheduled', 'reply', 'manual')),
    success INTEGER NOT NULL DEFAULT 1,
    error_msg TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引：按时间查询
CREATE INDEX IF NOT EXISTS idx_like_history_created_at ON like_history(created_at);
-- 索引：按用户查询
CREATE INDEX IF NOT EXISTS idx_like_history_user_id ON like_history(user_id);
-- 复合索引：查询用户当日是否已赞
CREATE INDEX IF NOT EXISTS idx_like_history_user_date ON like_history(user_id, created_at);

-- 新增配置默认值
INSERT OR IGNORE INTO config (key, value) VALUES ('reserved_for_reply', '10');
INSERT OR IGNORE INTO config (key, value) VALUES ('batch_interval', '3');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_times', '10');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_delay_min', '0');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_delay_max', '0');
