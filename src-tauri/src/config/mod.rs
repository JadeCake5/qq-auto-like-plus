use rusqlite::Connection;

use crate::db::models::{self, ConfigEntry};
use crate::errors::AppError;

pub fn get_all_config(conn: &Connection) -> Result<Vec<ConfigEntry>, AppError> {
    models::get_all_config(conn)
}

pub fn update_config(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    models::upsert_config(conn, key, value)
}
