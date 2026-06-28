use libsqlite3_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use crate::error::ArchivalError;

/// Execute a SQL statement with optional parameter binding. No rows returned.
pub unsafe fn execute(
    db: *mut sqlite3,
    sql: &str,
    bind: impl FnOnce(*mut sqlite3_stmt) -> Result<(), ArchivalError>,
) -> Result<(), ArchivalError> {
    let sql_c = CString::new(sql).map_err(|e| ArchivalError::InvalidInput(e.to_string()))?;
    let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();

    let rc = sqlite3_prepare_v2(db, sql_c.as_ptr(), -1, &mut stmt, std::ptr::null_mut());
    if rc != SQLITE_OK {
        let msg = errmsg(db);
        return Err(ArchivalError::Db(format!("prepare: {msg}")));
    }

    let bind_result = bind(stmt);
    if let Err(e) = bind_result {
        sqlite3_finalize(stmt);
        return Err(e);
    }

    let rc = sqlite3_step(stmt);
    sqlite3_finalize(stmt);

    if rc != SQLITE_DONE && rc != SQLITE_ROW {
        let msg = errmsg(db);
        return Err(ArchivalError::Db(format!("step: {msg}")));
    }
    Ok(())
}

/// Query rows, calling `on_row` for each. `on_row` receives the stmt for column extraction.
pub unsafe fn query<F>(
    db: *mut sqlite3,
    sql: &str,
    bind: impl FnOnce(*mut sqlite3_stmt) -> Result<(), ArchivalError>,
    mut on_row: F,
) -> Result<(), ArchivalError>
where
    F: FnMut(*mut sqlite3_stmt) -> Result<(), ArchivalError>,
{
    let sql_c = CString::new(sql).map_err(|e| ArchivalError::InvalidInput(e.to_string()))?;
    let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();

    let rc = sqlite3_prepare_v2(db, sql_c.as_ptr(), -1, &mut stmt, std::ptr::null_mut());
    if rc != SQLITE_OK {
        return Err(ArchivalError::Db(format!("prepare: {}", errmsg(db))));
    }

    if let Err(e) = bind(stmt) {
        sqlite3_finalize(stmt);
        return Err(e);
    }

    loop {
        let rc = sqlite3_step(stmt);
        match rc {
            SQLITE_ROW => {
                if let Err(e) = on_row(stmt) {
                    sqlite3_finalize(stmt);
                    return Err(e);
                }
            }
            SQLITE_DONE => break,
            _ => {
                let msg = errmsg(db);
                sqlite3_finalize(stmt);
                return Err(ArchivalError::Db(format!("step: {msg}")));
            }
        }
    }

    sqlite3_finalize(stmt);
    Ok(())
}

/// Execute a raw SQL string with no parameters (for multi-statement DDL).
pub unsafe fn exec_raw(db: *mut sqlite3, sql: &str) -> Result<(), ArchivalError> {
    let sql_c = CString::new(sql).map_err(|e| ArchivalError::InvalidInput(e.to_string()))?;
    let rc = sqlite3_exec(db, sql_c.as_ptr(), None, std::ptr::null_mut(), std::ptr::null_mut());
    if rc != SQLITE_OK {
        return Err(ArchivalError::Db(format!("exec: {}", errmsg(db))));
    }
    Ok(())
}

pub unsafe fn col_text(stmt: *mut sqlite3_stmt, col: c_int) -> Option<String> {
    let ptr = sqlite3_column_text(stmt, col) as *const c_char;
    if ptr.is_null() {
        None
    } else {
        Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

pub unsafe fn col_i64(stmt: *mut sqlite3_stmt, col: c_int) -> i64 {
    sqlite3_column_int64(stmt, col)
}

pub unsafe fn bind_text(stmt: *mut sqlite3_stmt, col: c_int, val: &str) -> Result<(), ArchivalError> {
    let c = CString::new(val).map_err(|e| ArchivalError::InvalidInput(e.to_string()))?;
    let rc = sqlite3_bind_text(stmt, col, c.as_ptr(), -1, SQLITE_TRANSIENT());
    if rc != SQLITE_OK {
        return Err(ArchivalError::Db(format!("bind_text col {col}")));
    }
    Ok(())
}

pub unsafe fn bind_text_opt(stmt: *mut sqlite3_stmt, col: c_int, val: Option<&str>) -> Result<(), ArchivalError> {
    match val {
        Some(v) => bind_text(stmt, col, v),
        None => {
            let rc = sqlite3_bind_null(stmt, col);
            if rc != SQLITE_OK {
                return Err(ArchivalError::Db(format!("bind_null col {col}")));
            }
            Ok(())
        }
    }
}

pub unsafe fn bind_i64(stmt: *mut sqlite3_stmt, col: c_int, val: i64) -> Result<(), ArchivalError> {
    let rc = sqlite3_bind_int64(stmt, col, val);
    if rc != SQLITE_OK {
        return Err(ArchivalError::Db(format!("bind_i64 col {col}")));
    }
    Ok(())
}

unsafe fn errmsg(db: *mut sqlite3) -> String {
    let ptr = sqlite3_errmsg(db);
    if ptr.is_null() {
        "unknown error".to_string()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

// Helper to get SQLITE_TRANSIENT as a function pointer
#[allow(non_snake_case)]
unsafe fn SQLITE_TRANSIENT() -> Option<unsafe extern "C" fn(*mut std::ffi::c_void)> {
    // SQLITE_TRANSIENT = -1 cast to a function pointer
    std::mem::transmute(-1isize)
}
