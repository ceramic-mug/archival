use libsqlite3_sys::*;
use crate::error::ArchivalError;
use super::ops::{exec_raw, query, col_i64};

const MIGRATION_V1: &str = include_str!("../../migrations/v1.sql");
const MIGRATION_V2: &str = include_str!("../../migrations/v2.sql");

pub unsafe fn initialize(db: *mut sqlite3) -> Result<(), ArchivalError> {
    // WAL mode for iCloud compatibility
    exec_raw(db, "PRAGMA journal_mode=WAL;")?;
    exec_raw(db, "PRAGMA foreign_keys=ON;")?;

    let current_version = get_schema_version(db)?;

    if current_version < 1 {
        apply_v1(db)?;
    }
    if current_version < 2 {
        apply_v2(db)?;
    }

    Ok(())
}

unsafe fn get_schema_version(db: *mut sqlite3) -> Result<i64, ArchivalError> {
    // Check if schema_version table exists
    let mut exists = false;
    query(
        db,
        "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version'",
        |_| Ok(()),
        |_stmt| { exists = true; Ok(()) },
    )?;

    if !exists {
        return Ok(0);
    }

    let mut version: i64 = 0;
    query(
        db,
        "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
        |_| Ok(()),
        |stmt| { version = col_i64(stmt, 0); Ok(()) },
    )?;
    Ok(version)
}

unsafe fn apply_v1(db: *mut sqlite3) -> Result<(), ArchivalError> {
    exec_raw(db, MIGRATION_V1)
}

unsafe fn apply_v2(db: *mut sqlite3) -> Result<(), ArchivalError> {
    exec_raw(db, MIGRATION_V2)
}
