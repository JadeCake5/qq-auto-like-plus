-- 标签策略扩展
ALTER TABLE tags ADD COLUMN like_times INTEGER DEFAULT NULL;
ALTER TABLE tags ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium';
ALTER TABLE tags ADD COLUMN auto_like INTEGER NOT NULL DEFAULT 1;
ALTER TABLE tags ADD COLUMN auto_reply INTEGER NOT NULL DEFAULT 1;

-- 系统标签默认策略
UPDATE tags SET auto_like = 0, auto_reply = 0 WHERE name = '不赞' AND is_system = 1;
UPDATE tags SET priority = 'high' WHERE name = '重要' AND is_system = 1;
