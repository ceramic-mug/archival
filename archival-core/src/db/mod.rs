pub mod ops;
pub mod schema;

use libsqlite3_sys::sqlite3;
use std::ffi::CString;

pub struct DbHandle {
    pub(crate) db: *mut sqlite3,
    pub(crate) media_root: CString,
}

// Safety: caller is responsible for single-threaded access (documented in header).
unsafe impl Send for DbHandle {}
unsafe impl Sync for DbHandle {}

impl Drop for DbHandle {
    fn drop(&mut self) {
        if !self.db.is_null() {
            unsafe { libsqlite3_sys::sqlite3_close(self.db) };
            self.db = std::ptr::null_mut();
        }
    }
}
