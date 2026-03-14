pub mod migrations;
pub mod models;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type DbState = Arc<Mutex<Connection>>;

pub fn init_db(app_data_dir: &std::path::Path) -> Result<DbState, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("data.db");
    let conn = Connection::open(&db_path)?;

    // 启用 WAL 模式
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    // 执行迁移
    migrations::run_migrations(&conn)?;

    tracing::info!("数据库初始化完成: {:?}", db_path);
    Ok(Arc::new(Mutex::new(conn)))
}
