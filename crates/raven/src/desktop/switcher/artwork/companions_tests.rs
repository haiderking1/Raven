use std::ffi::{CString, c_char, c_int, c_void};
unsafe extern "C" {
    fn g_key_file_new() -> *mut c_void;
    fn g_key_file_load_from_data(
        file: *mut c_void,
        data: *const c_char,
        len: usize,
        flags: c_int,
        error: *mut *mut c_void,
    ) -> c_int;
    fn g_key_file_unref(file: *mut c_void);
    fn g_desktop_app_info_new_from_keyfile(file: *mut c_void) -> *mut c_void;
    fn g_object_unref(object: *mut c_void);
    fn g_list_append(list: *mut c_void, data: *mut c_void) -> *mut c_void;
    fn g_list_free(list: *mut c_void);
    fn g_app_info_get_icon(app: *mut c_void) -> *mut c_void;
    fn g_icon_equal(first: *mut c_void, second: *mut c_void) -> c_int;
    fn raven_companion_icon(apps: *mut c_void, selected: *mut c_void) -> *mut c_void;
}
struct App(*mut c_void);
impl App {
    fn new(name: &str, types: &str, icon: Option<&str>) -> Self {
        let icon = icon.map(|i| format!("Icon={i}\n")).unwrap_or_default();
        let text = CString::new(format!("[Desktop Entry]\nType=Application\nName={name}\nExec=/bin/false\nMimeType={types};\n{icon}")).unwrap();
        unsafe {
            let file = g_key_file_new();
            assert_ne!(
                g_key_file_load_from_data(
                    file,
                    text.as_ptr(),
                    text.as_bytes().len(),
                    0,
                    std::ptr::null_mut()
                ),
                0
            );
            let app = g_desktop_app_info_new_from_keyfile(file);
            g_key_file_unref(file);
            assert!(!app.is_null());
            Self(app)
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            g_object_unref(self.0);
        }
    }
}
fn companion(selected: &App, candidates: &[&App]) -> *mut c_void {
    unsafe {
        let mut list = std::ptr::null_mut();
        for app in candidates {
            list = g_list_append(list, app.0);
        }
        let icon = raven_companion_icon(list, selected.0);
        g_list_free(list);
        icon
    }
}
#[test]
fn generated_handler_inherits_the_packaged_icon_without_executing_its_wrapper() {
    let helper = App::new("Editor (Nightly)", "x-scheme-handler/editor", None);
    let packaged = App::new(
        "Editor Nightly",
        "x-scheme-handler/editor",
        Some("editor-nightly"),
    );
    let unrelated = App::new("Other Editor", "x-scheme-handler/editor", Some("other"));
    let icon = companion(&helper, &[&unrelated, &packaged]);
    assert!(!icon.is_null());
    assert_ne!(
        unsafe { g_icon_equal(icon, g_app_info_get_icon(packaged.0)) },
        0
    );
}
#[test]
fn a_similar_name_alone_and_conflicting_companions_are_not_icon_matches() {
    let helper = App::new("Editor (Nightly)", "x-scheme-handler/editor", None);
    let unrelated = App::new("Editor Nightly", "text/plain", Some("unrelated"));
    assert!(companion(&helper, &[&unrelated]).is_null());
    let first = App::new("Editor Nightly", "x-scheme-handler/editor", Some("first"));
    let second = App::new("Editor Nightly", "x-scheme-handler/editor", Some("second"));
    assert!(companion(&helper, &[&first, &second]).is_null());
}
