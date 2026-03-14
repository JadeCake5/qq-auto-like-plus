use rusqlite::Connection;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../../migrations/001_init.sql")),
    ("002_quota_and_history", include_str!("../../migrations/002_quota_and_history.sql")),
    ("003_friends", include_str!("../../migrations/003_friends.sql")),
    ("004_scheduler_config", include_str!("../../migrations/004_scheduler_config.sql")),
    ("005_webhook_config", include_str!("../../migrations/005_webhook_config.sql")),
    ("006_reply_config", include_str!("../../migrations/006_reply_config.sql")),
    ("007_tags", include_str!("../../migrations/007_tags.sql")),
    ("008_tag_strategy", include_str!("../../migrations/008_tag_strategy.sql")),
];

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    for (name, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;

        if !applied {
            conn.execute_batch(&format!("BEGIN; {} COMMIT;", sql))
                .or_else(|e| {
                    let _ = conn.execute_batch("ROLLBACK;");
                    Err(e)
                })?;
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
            tracing::info!("Applied migration: {}", name);
        }
    }
    Ok(())
}
