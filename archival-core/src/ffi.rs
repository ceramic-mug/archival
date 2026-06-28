use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::time::{SystemTime, UNIX_EPOCH};

use libsqlite3_sys::*;
use uuid::Uuid;

use crate::categories::definitions::find_category;
use crate::db::{DbHandle, ops::*};
use crate::db::schema::initialize;
use crate::error::{ArchivalError, ArchivalResult, last_error_ptr, set_last_error};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

unsafe fn cstr<'a>(ptr: *const c_char) -> Result<&'a str, ArchivalError> {
    if ptr.is_null() {
        return Err(ArchivalError::InvalidInput("null pointer".into()));
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|e| ArchivalError::InvalidInput(e.to_string()))
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// Allocate a CString and return raw pointer (caller must free with archival_free_string).
fn to_out_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn handle_result(result: Result<(), ArchivalError>) -> ArchivalResult {
    match result {
        Ok(()) => ArchivalResult::Ok,
        Err(e) => e.record_and_return(),
    }
}

// ─── Lifecycle ────────────────────────────────────────────────────────────────

/// Open (or create) an Archival database.
/// Returns NULL on failure; call archival_last_error() for details.
/// db_path and media_root must be valid UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn archival_db_open(
    db_path: *const c_char,
    media_root: *const c_char,
) -> *mut DbHandle {
    let path = match cstr(db_path) {
        Ok(s) => s,
        Err(e) => { set_last_error(e.message()); return std::ptr::null_mut(); }
    };
    let root = match cstr(media_root) {
        Ok(s) => s,
        Err(e) => { set_last_error(e.message()); return std::ptr::null_mut(); }
    };

    let path_c = match CString::new(path) {
        Ok(c) => c,
        Err(e) => { set_last_error(&e.to_string()); return std::ptr::null_mut(); }
    };

    let mut db: *mut sqlite3 = std::ptr::null_mut();
    let rc = sqlite3_open_v2(
        path_c.as_ptr(),
        &mut db,
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
        std::ptr::null(),
    );

    if rc != SQLITE_OK {
        set_last_error(&format!("sqlite3_open_v2 failed: {rc}"));
        if !db.is_null() { sqlite3_close(db); }
        return std::ptr::null_mut();
    }

    let media_root_c = match CString::new(root) {
        Ok(c) => c,
        Err(e) => { set_last_error(&e.to_string()); sqlite3_close(db); return std::ptr::null_mut(); }
    };

    let handle = Box::new(DbHandle { db, media_root: media_root_c });

    if let Err(e) = initialize(handle.db) {
        set_last_error(e.message());
        return std::ptr::null_mut();
    }

    Box::into_raw(handle)
}

/// Close the database and free the handle. Safe to call with NULL.
#[no_mangle]
pub unsafe extern "C" fn archival_db_close(handle: *mut DbHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Return the last error message (thread-local). Valid until next call on this thread.
#[no_mangle]
pub unsafe extern "C" fn archival_last_error() -> *const c_char {
    last_error_ptr()
}

/// Free a string returned by any archival_* function that outputs a string.
#[no_mangle]
pub unsafe extern "C" fn archival_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// ─── Items ────────────────────────────────────────────────────────────────────

/// Create a new item of the given category. On success, *out_id is set to a
/// newly allocated UUID string (caller must free with archival_free_string).
#[no_mangle]
pub unsafe extern "C" fn archival_item_create(
    handle: *mut DbHandle,
    category: *const c_char,
    out_id: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;
    let cat = match cstr(category) {
        Ok(s) => s,
        Err(e) => return e.record_and_return(),
    };

    let cat_def = match find_category(cat) {
        Some(c) => c,
        None => return ArchivalError::InvalidInput(format!("unknown category: {cat}")).record_and_return(),
    };

    let id = new_id();
    let ts = now_unix();

    let result: Result<(), ArchivalError> = (|| {
        execute(h.db, "BEGIN", |_| Ok(()))?;

        execute(
            h.db,
            "INSERT INTO items(id, category, created_at, updated_at) VALUES(?1,?2,?3,?4)",
            |stmt| {
                bind_text(stmt, 1, &id)?;
                bind_text(stmt, 2, cat)?;
                bind_i64(stmt, 3, ts)?;
                bind_i64(stmt, 4, ts)?;
                Ok(())
            },
        )?;

        let field_sql = format!(
            "INSERT INTO {}(item_id) VALUES(?1)",
            cat_def.table
        );
        execute(h.db, &field_sql, |stmt| {
            bind_text(stmt, 1, &id)?;
            Ok(())
        })?;

        execute(h.db, "COMMIT", |_| Ok(()))?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            if !out_id.is_null() {
                *out_id = to_out_str(id);
            }
            ArchivalResult::Ok
        }
        Err(e) => {
            let _ = execute(h.db, "ROLLBACK", |_| Ok(()));
            e.record_and_return()
        }
    }
}

/// Delete an item and all associated data (cascades via FK).
#[no_mangle]
pub unsafe extern "C" fn archival_item_delete(
    handle: *mut DbHandle,
    item_id: *const c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id = match cstr(item_id) { Ok(s) => s, Err(e) => return e.record_and_return() };
    handle_result(execute(h.db, "DELETE FROM items WHERE id=?1", |stmt| {
        bind_text(stmt, 1, id)
    }))
}

/// Set fields for an item from a JSON object string.
/// Keys must match sql_column names for the item's category.
/// Unknown keys are ignored. Existing values are overwritten.
#[no_mangle]
pub unsafe extern "C" fn archival_item_set_fields(
    handle: *mut DbHandle,
    item_id: *const c_char,
    fields_json: *const c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id = match cstr(item_id) { Ok(s) => s, Err(e) => return e.record_and_return() };
    let json = match cstr(fields_json) { Ok(s) => s, Err(e) => return e.record_and_return() };

    let result: Result<(), ArchivalError> = (|| {
        // Fetch category for this item
        let mut cat_name = String::new();
        query(
            h.db,
            "SELECT category FROM items WHERE id=?1",
            |stmt| bind_text(stmt, 1, id),
            |stmt| { cat_name = col_text(stmt, 0).unwrap_or_default(); Ok(()) },
        )?;

        if cat_name.is_empty() {
            return Err(ArchivalError::NotFound(format!("item not found: {id}")));
        }

        let cat_def = find_category(&cat_name)
            .ok_or_else(|| ArchivalError::InvalidInput(format!("unknown category: {cat_name}")))?;

        // Parse JSON minimally — extract key/value pairs
        let kvs = parse_json_object(json)
            .map_err(|e| ArchivalError::Json(e))?;

        // Build SET clause from known fields only
        let mut set_parts: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for (k, v) in &kvs {
            if cat_def.fields.iter().any(|f| f.sql_column == k.as_str()) {
                set_parts.push(format!("{k}=?{}", set_parts.len() + 2));
                values.push(v.clone());
            }
        }

        if set_parts.is_empty() {
            return Ok(()); // nothing to update
        }

        // Update field table
        let sql = format!(
            "UPDATE {} SET {} WHERE item_id=?1",
            cat_def.table,
            set_parts.join(", ")
        );

        execute(h.db, &sql, |stmt| {
            bind_text(stmt, 1, id)?;
            for (i, v) in values.iter().enumerate() {
                if v == "null" {
                    bind_text_opt(stmt, (i + 2) as i32, None)?;
                } else {
                    bind_text(stmt, (i + 2) as i32, v)?;
                }
            }
            Ok(())
        })?;

        // Derive display_name from the most descriptive field present (skip null values)
        let display_name = ["title", "name", "subject"].iter()
            .find_map(|key| {
                kvs.iter()
                    .find(|(k, v)| k.as_str() == *key && v != "null" && !v.is_empty())
                    .map(|(_, v)| v.clone())
            });

        execute(h.db, "UPDATE items SET updated_at=?1, display_name=?2 WHERE id=?3", |stmt| {
            bind_i64(stmt, 1, now_unix())?;
            bind_text_opt(stmt, 2, display_name.as_deref())?;
            bind_text(stmt, 3, id)?;
            Ok(())
        })?;

        Ok(())
    })();

    handle_result(result)
}

/// Get fields for an item as a JSON object string. Caller must free with archival_free_string.
#[no_mangle]
pub unsafe extern "C" fn archival_item_get_fields(
    handle: *mut DbHandle,
    item_id: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id = match cstr(item_id) { Ok(s) => s, Err(e) => return e.record_and_return() };

    let result: Result<String, ArchivalError> = (|| {
        let mut cat_name = String::new();
        query(
            h.db,
            "SELECT category FROM items WHERE id=?1",
            |stmt| bind_text(stmt, 1, id),
            |stmt| { cat_name = col_text(stmt, 0).unwrap_or_default(); Ok(()) },
        )?;

        if cat_name.is_empty() {
            return Err(ArchivalError::NotFound(format!("item not found: {id}")));
        }

        let cat_def = find_category(&cat_name)
            .ok_or_else(|| ArchivalError::InvalidInput(format!("unknown category: {cat_name}")))?;

        let cols: Vec<&str> = cat_def.fields.iter().map(|f| f.sql_column).collect();
        let select = cols.join(", ");
        let sql = format!("SELECT {select} FROM {} WHERE item_id=?1", cat_def.table);

        let mut json = String::from("{");
        query(
            h.db,
            &sql,
            |stmt| bind_text(stmt, 1, id),
            |stmt| {
                for (i, col) in cols.iter().enumerate() {
                    if i > 0 { json.push(','); }
                    json.push('"');
                    json.push_str(col);
                    json.push_str("\":");
                    match col_text(stmt, i as i32) {
                        Some(v) => { json.push('"'); json.push_str(&v.replace('"', "\\\"")); json.push('"'); }
                        None => json.push_str("null"),
                    }
                }
                Ok(())
            },
        )?;
        json.push('}');
        Ok(json)
    })();

    match result {
        Ok(json) => {
            if !out_json.is_null() {
                *out_json = to_out_str(json);
            }
            ArchivalResult::Ok
        }
        Err(e) => e.record_and_return(),
    }
}

/// List items, optionally filtered by category. Returns JSON array of item summaries.
/// Each element: {"id":"...","category":"...","created_at":N,"updated_at":N,"notes":...}
#[no_mangle]
pub unsafe extern "C" fn archival_items_list(
    handle: *mut DbHandle,
    category_filter: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;

    let filter = if category_filter.is_null() {
        None
    } else {
        match cstr(category_filter) {
            Ok(s) => Some(s.to_string()),
            Err(e) => return e.record_and_return(),
        }
    };

    let result: Result<String, ArchivalError> = (|| {
        let (sql, use_filter) = match &filter {
            Some(_) => ("SELECT id,category,created_at,updated_at,notes,display_name FROM items WHERE category=?1 ORDER BY created_at DESC", true),
            None    => ("SELECT id,category,created_at,updated_at,notes,display_name FROM items ORDER BY created_at DESC", false),
        };

        let mut json = String::from("[");
        let mut first = true;

        let bind_fn = |stmt: *mut sqlite3_stmt| -> Result<(), ArchivalError> {
            if use_filter {
                bind_text(stmt, 1, filter.as_deref().unwrap())?;
            }
            Ok(())
        };

        query(h.db, sql, bind_fn, |stmt| {
            if !first { json.push(','); }
            first = false;
            let id           = col_text(stmt, 0).unwrap_or_default();
            let cat          = col_text(stmt, 1).unwrap_or_default();
            let created      = col_i64(stmt, 2);
            let updated      = col_i64(stmt, 3);
            let notes        = col_text(stmt, 4);
            let display_name = col_text(stmt, 5);
            let notes_json = match notes {
                Some(n) => format!(r#""{}""#, n.replace('"', "\\\"")),
                None    => "null".into(),
            };
            let dn_json = match display_name {
                Some(n) => format!(r#""{}""#, n.replace('"', "\\\"")),
                None    => "null".into(),
            };
            json.push_str(&format!(
                r#"{{"id":"{id}","category":"{cat}","created_at":{created},"updated_at":{updated},"notes":{notes_json},"display_name":{dn_json}}}"#
            ));
            Ok(())
        })?;

        json.push(']');
        Ok(json)
    })();

    match result {
        Ok(json) => { if !out_json.is_null() { *out_json = to_out_str(json); } ArchivalResult::Ok }
        Err(e) => e.record_and_return(),
    }
}

// ─── Media ────────────────────────────────────────────────────────────────────

/// Add a media file reference to an item. file_path is relative to media_root.
#[no_mangle]
pub unsafe extern "C" fn archival_media_add(
    handle: *mut DbHandle,
    item_id: *const c_char,
    relative_path: *const c_char,
    mime_type: *const c_char,
    is_primary: i32,
) -> ArchivalResult {
    let h = &*handle;
    let id   = match cstr(item_id)       { Ok(s) => s, Err(e) => return e.record_and_return() };
    let path = match cstr(relative_path) { Ok(s) => s, Err(e) => return e.record_and_return() };
    let mime = match cstr(mime_type)     { Ok(s) => s, Err(e) => return e.record_and_return() };
    let media_id = new_id();
    let ts = now_unix();
    handle_result(execute(
        h.db,
        "INSERT INTO media_files(id,item_id,file_path,mime_type,is_primary,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
        |stmt| {
            bind_text(stmt, 1, &media_id)?;
            bind_text(stmt, 2, id)?;
            bind_text(stmt, 3, path)?;
            bind_text(stmt, 4, mime)?;
            bind_i64(stmt, 5, is_primary as i64)?;
            bind_i64(stmt, 6, ts)?;
            Ok(())
        },
    ))
}

/// List media files for an item. Returns JSON array.
#[no_mangle]
pub unsafe extern "C" fn archival_media_list(
    handle: *mut DbHandle,
    item_id: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id = match cstr(item_id) { Ok(s) => s, Err(e) => return e.record_and_return() };

    let result: Result<String, ArchivalError> = (|| {
        let mut json = String::from("[");
        let mut first = true;
        query(
            h.db,
            "SELECT id,file_path,mime_type,is_primary,created_at FROM media_files WHERE item_id=?1",
            |stmt| bind_text(stmt, 1, id),
            |stmt| {
                if !first { json.push(','); }
                first = false;
                let mid   = col_text(stmt, 0).unwrap_or_default();
                let path  = col_text(stmt, 1).unwrap_or_default();
                let mime  = col_text(stmt, 2).unwrap_or_default();
                let prim  = col_i64(stmt, 3);
                let cat   = col_i64(stmt, 4);
                json.push_str(&format!(
                    r#"{{"id":"{mid}","file_path":"{path}","mime_type":"{mime}","is_primary":{prim},"created_at":{cat}}}"#
                ));
                Ok(())
            },
        )?;
        json.push(']');
        Ok(json)
    })();

    match result {
        Ok(j) => { if !out_json.is_null() { *out_json = to_out_str(j); } ArchivalResult::Ok }
        Err(e) => e.record_and_return(),
    }
}

// ─── Tags ─────────────────────────────────────────────────────────────────────

/// Add a tag to an item (creates the tag if it doesn't exist).
#[no_mangle]
pub unsafe extern "C" fn archival_tag_add(
    handle: *mut DbHandle,
    item_id: *const c_char,
    tag_name: *const c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id  = match cstr(item_id)  { Ok(s) => s, Err(e) => return e.record_and_return() };
    let tag = match cstr(tag_name) { Ok(s) => s, Err(e) => return e.record_and_return() };

    let tag_id = new_id();
    handle_result((|| {
        execute(h.db, "BEGIN", |_| Ok(()))?;
        execute(
            h.db,
            "INSERT OR IGNORE INTO tags(id,name) VALUES(?1,?2)",
            |stmt| { bind_text(stmt, 1, &tag_id)?; bind_text(stmt, 2, tag) },
        )?;
        execute(
            h.db,
            "INSERT OR IGNORE INTO item_tags(item_id,tag_id) SELECT ?1,id FROM tags WHERE name=?2",
            |stmt| { bind_text(stmt, 1, id)?; bind_text(stmt, 2, tag) },
        )?;
        execute(h.db, "COMMIT", |_| Ok(()))?;
        Ok(())
    })())
}

/// Remove a tag from an item.
#[no_mangle]
pub unsafe extern "C" fn archival_tag_remove(
    handle: *mut DbHandle,
    item_id: *const c_char,
    tag_name: *const c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id  = match cstr(item_id)  { Ok(s) => s, Err(e) => return e.record_and_return() };
    let tag = match cstr(tag_name) { Ok(s) => s, Err(e) => return e.record_and_return() };
    handle_result(execute(
        h.db,
        "DELETE FROM item_tags WHERE item_id=?1 AND tag_id=(SELECT id FROM tags WHERE name=?2)",
        |stmt| { bind_text(stmt, 1, id)?; bind_text(stmt, 2, tag) },
    ))
}

/// Get tags for an item. Returns JSON array of tag name strings.
#[no_mangle]
pub unsafe extern "C" fn archival_tags_for_item(
    handle: *mut DbHandle,
    item_id: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id = match cstr(item_id) { Ok(s) => s, Err(e) => return e.record_and_return() };

    let result: Result<String, ArchivalError> = (|| {
        let mut json = String::from("[");
        let mut first = true;
        query(
            h.db,
            "SELECT t.name FROM tags t JOIN item_tags it ON t.id=it.tag_id WHERE it.item_id=?1",
            |stmt| bind_text(stmt, 1, id),
            |stmt| {
                if !first { json.push(','); }
                first = false;
                let name = col_text(stmt, 0).unwrap_or_default();
                json.push('"'); json.push_str(&name); json.push('"');
                Ok(())
            },
        )?;
        json.push(']');
        Ok(json)
    })();

    match result {
        Ok(j) => { if !out_json.is_null() { *out_json = to_out_str(j); } ArchivalResult::Ok }
        Err(e) => e.record_and_return(),
    }
}

// ─── AI Pipeline ──────────────────────────────────────────────────────────────

/// Classify an image. Returns JSON: {"category":"Book"}.
/// Caller must free out_json with archival_free_string.
#[no_mangle]
pub unsafe extern "C" fn archival_ai_classify(
    _handle: *mut DbHandle,
    image_path: *const c_char,
    api_key: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let path = match cstr(image_path) { Ok(s) => s, Err(e) => return e.record_and_return() };
    let key  = match cstr(api_key)    { Ok(s) => s, Err(e) => return e.record_and_return() };

    match crate::ai::classifier::classify_image(path, key) {
        Ok(json) => { if !out_json.is_null() { *out_json = to_out_str(json); } ArchivalResult::Ok }
        Err(e) => e.record_and_return(),
    }
}

/// Fill category fields from an image via the specialist agent.
/// Returns JSON object of field values. Caller must free with archival_free_string.
#[no_mangle]
pub unsafe extern "C" fn archival_ai_fill_fields(
    handle: *mut DbHandle,
    item_id: *const c_char,
    image_path: *const c_char,
    api_key: *const c_char,
    out_json: *mut *mut c_char,
) -> ArchivalResult {
    let h = &*handle;
    let id   = match cstr(item_id)    { Ok(s) => s, Err(e) => return e.record_and_return() };
    let path = match cstr(image_path) { Ok(s) => s, Err(e) => return e.record_and_return() };
    let key  = match cstr(api_key)    { Ok(s) => s, Err(e) => return e.record_and_return() };

    let result: Result<String, ArchivalError> = (|| {
        let mut cat_name = String::new();
        query(
            h.db,
            "SELECT category FROM items WHERE id=?1",
            |stmt| bind_text(stmt, 1, id),
            |stmt| { cat_name = col_text(stmt, 0).unwrap_or_default(); Ok(()) },
        )?;
        if cat_name.is_empty() {
            return Err(ArchivalError::NotFound(format!("item not found: {id}")));
        }
        crate::ai::specialist::fill_fields(path, &cat_name, key)
    })();

    match result {
        Ok(json) => { if !out_json.is_null() { *out_json = to_out_str(json); } ArchivalResult::Ok }
        Err(e) => e.record_and_return(),
    }
}

// ─── Minimal JSON parser ──────────────────────────────────────────────────────

/// Parse a flat JSON object {"key":"value",...} into key-value pairs.
/// Values are returned as raw strings (with surrounding quotes stripped).
/// Nested objects/arrays not supported — field values are always strings or null.
fn parse_json_object(json: &str) -> Result<Vec<(String, String)>, String> {
    let json = json.trim();
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err("not a JSON object".into());
    }
    let inner = &json[1..json.len() - 1];
    let mut pairs = Vec::new();

    // Very simple tokenizer: find "key":"value" or "key":null patterns
    let mut chars = inner.chars().peekable();

    fn skip_ws(c: &mut std::iter::Peekable<std::str::Chars>) {
        while matches!(c.peek(), Some(' ' | '\t' | '\n' | '\r')) { c.next(); }
    }

    fn read_string(c: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
        if c.next() != Some('"') { return Err("expected '\"'".into()); }
        let mut s = String::new();
        let mut escaped = false;
        for ch in c.by_ref() {
            if escaped { s.push(ch); escaped = false; }
            else if ch == '\\' { escaped = true; }
            else if ch == '"' { return Ok(s); }
            else { s.push(ch); }
        }
        Err("unterminated string".into())
    }

    loop {
        skip_ws(&mut chars);
        if chars.peek().is_none() { break; }

        let key = read_string(&mut chars)?;
        skip_ws(&mut chars);
        if chars.next() != Some(':') { return Err("expected ':'".into()); }
        skip_ws(&mut chars);

        let value = match chars.peek() {
            Some('"') => read_string(&mut chars)?,
            Some('n') => {
                // consume "null"
                for _ in 0..4 { chars.next(); }
                "null".into()
            }
            _ => {
                // consume until ',' or '}'
                let mut v = String::new();
                while !matches!(chars.peek(), Some(',' | '}') | None) {
                    v.push(chars.next().unwrap());
                }
                v.trim().to_string()
            }
        };

        pairs.push((key, value));
        skip_ws(&mut chars);
        match chars.peek() {
            Some(',') => { chars.next(); }
            _ => break,
        }
    }

    Ok(pairs)
}
