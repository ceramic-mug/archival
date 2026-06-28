pub mod ai;
pub mod categories;
pub mod db;
pub mod error;
pub mod ffi;

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use crate::ffi::*;
    use crate::error::ArchivalResult;

    fn tmp_path() -> (CString, CString, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path  = CString::new(dir.path().join("test.db").to_str().unwrap()).unwrap();
        let media_root = CString::new(dir.path().join("media").to_str().unwrap()).unwrap();
        (db_path, media_root, dir)
    }

    #[test]
    fn test_open_close() {
        let (db_path, media_root, _dir) = tmp_path();
        let h = unsafe { archival_db_open(db_path.as_ptr(), media_root.as_ptr()) };
        assert!(!h.is_null(), "db open failed");
        unsafe { archival_db_close(h) };
    }

    #[test]
    fn test_item_roundtrip() {
        let (db_path, media_root, _dir) = tmp_path();
        let h = unsafe { archival_db_open(db_path.as_ptr(), media_root.as_ptr()) };
        assert!(!h.is_null());

        let cat = CString::new("Book").unwrap();
        let mut out_id: *mut std::os::raw::c_char = std::ptr::null_mut();

        let rc = unsafe { archival_item_create(h, cat.as_ptr(), &mut out_id) };
        assert_eq!(rc, ArchivalResult::Ok);
        assert!(!out_id.is_null());

        let id = unsafe { std::ffi::CStr::from_ptr(out_id).to_str().unwrap().to_string() };
        let id_c = CString::new(id.clone()).unwrap();

        // Set fields
        let fields = CString::new(r#"{"title":"Moby Dick","author":"Herman Melville"}"#).unwrap();
        let rc = unsafe { archival_item_set_fields(h, id_c.as_ptr(), fields.as_ptr()) };
        assert_eq!(rc, ArchivalResult::Ok);

        // Get fields back
        let mut out_json: *mut std::os::raw::c_char = std::ptr::null_mut();
        let rc = unsafe { archival_item_get_fields(h, id_c.as_ptr(), &mut out_json) };
        assert_eq!(rc, ArchivalResult::Ok);
        assert!(!out_json.is_null());

        let json_str = unsafe { std::ffi::CStr::from_ptr(out_json).to_str().unwrap() };
        assert!(json_str.contains("Moby Dick"), "fields not persisted: {json_str}");
        assert!(json_str.contains("Herman Melville"));

        unsafe { archival_free_string(out_json) };

        // List items
        let mut list_json: *mut std::os::raw::c_char = std::ptr::null_mut();
        let rc = unsafe { archival_items_list(h, std::ptr::null(), &mut list_json) };
        assert_eq!(rc, ArchivalResult::Ok);
        let list_str = unsafe { std::ffi::CStr::from_ptr(list_json).to_str().unwrap() };
        assert!(list_str.contains(&id));
        unsafe { archival_free_string(list_json) };

        // Tags
        let tag = CString::new("childhood").unwrap();
        unsafe { archival_tag_add(h, id_c.as_ptr(), tag.as_ptr()) };
        let mut tags_json: *mut std::os::raw::c_char = std::ptr::null_mut();
        unsafe { archival_tags_for_item(h, id_c.as_ptr(), &mut tags_json) };
        let tags_str = unsafe { std::ffi::CStr::from_ptr(tags_json).to_str().unwrap() };
        assert!(tags_str.contains("childhood"), "tag not found: {tags_str}");
        unsafe { archival_free_string(tags_json) };

        // Delete
        let rc = unsafe { archival_item_delete(h, id_c.as_ptr()) };
        assert_eq!(rc, ArchivalResult::Ok);

        unsafe { archival_free_string(out_id) };
        unsafe { archival_db_close(h) };
    }

    #[test]
    fn test_all_categories_create() {
        let (db_path, media_root, _dir) = tmp_path();
        let h = unsafe { archival_db_open(db_path.as_ptr(), media_root.as_ptr()) };
        assert!(!h.is_null());

        for cat_name in ["Book","Music","Movie","Game","PersonalMessage","Award",
                         "Art","Photograph","Trinket","Jewelry","Clothing","Object"] {
            let cat = CString::new(cat_name).unwrap();
            let mut out_id: *mut std::os::raw::c_char = std::ptr::null_mut();
            let rc = unsafe { archival_item_create(h, cat.as_ptr(), &mut out_id) };
            assert_eq!(rc, ArchivalResult::Ok, "failed to create {cat_name}");
            unsafe { archival_free_string(out_id) };
        }

        unsafe { archival_db_close(h) };
    }
}
