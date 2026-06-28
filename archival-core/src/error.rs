use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchivalResult {
    Ok = 0,
    ErrDb = 1,
    ErrNotFound = 2,
    ErrInvalidInput = 3,
    ErrIo = 4,
    ErrJson = 5,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

pub fn last_error_ptr() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow().as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
    })
}

#[derive(Debug)]
pub enum ArchivalError {
    Db(String),
    NotFound(String),
    InvalidInput(String),
    Io(String),
    Json(String),
}

impl ArchivalError {
    pub fn result_code(&self) -> ArchivalResult {
        match self {
            ArchivalError::Db(_) => ArchivalResult::ErrDb,
            ArchivalError::NotFound(_) => ArchivalResult::ErrNotFound,
            ArchivalError::InvalidInput(_) => ArchivalResult::ErrInvalidInput,
            ArchivalError::Io(_) => ArchivalResult::ErrIo,
            ArchivalError::Json(_) => ArchivalResult::ErrJson,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ArchivalError::Db(m) => m,
            ArchivalError::NotFound(m) => m,
            ArchivalError::InvalidInput(m) => m,
            ArchivalError::Io(m) => m,
            ArchivalError::Json(m) => m,
        }
    }

    pub fn record_and_return(&self) -> ArchivalResult {
        set_last_error(self.message());
        self.result_code()
    }
}
