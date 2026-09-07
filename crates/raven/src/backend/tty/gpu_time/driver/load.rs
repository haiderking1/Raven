//! Only typed, non-null EGL function pointers enter the driver.

use smithay::backend::{egl::get_proc_address, renderer::gles::ffi};
use std::ffi::{CStr, c_void};

pub(super) const TIME_ELAPSED: u32 = 0x88BF;
pub(super) const CURRENT_QUERY: u32 = 0x8865;
pub(super) const COUNTER_BITS: u32 = 0x8864;
pub(super) const RESULT: u32 = 0x8866;
pub(super) const AVAILABLE: u32 = 0x8867;
pub(super) const DISJOINT: u32 = 0x8FBB;

type Gen = unsafe extern "system" fn(i32, *mut u32);
type Delete = unsafe extern "system" fn(i32, *const u32);
type IsQuery = unsafe extern "system" fn(u32) -> u8;
type Begin = unsafe extern "system" fn(u32, u32);
type End = unsafe extern "system" fn(u32);
type GetQuery = unsafe extern "system" fn(u32, u32, *mut i32);
type GetUnsigned = unsafe extern "system" fn(u32, u32, *mut u32);
type GetSigned64 = unsafe extern "system" fn(u32, u32, *mut i64);
type GetUnsigned64 = unsafe extern "system" fn(u32, u32, *mut u64);
type GetInteger64 = unsafe extern "system" fn(u32, *mut i64);
type ResetStatus = unsafe extern "system" fn() -> u32;

pub(crate) struct Extension {
    pub(super) gen_queries: Gen,
    pub(super) delete_queries: Delete,
    pub(super) begin_query: Begin,
    pub(super) end_query: End,
    pub(super) get_query: GetQuery,
    pub(super) get_unsigned: GetUnsigned,
    pub(super) get_unsigned64: GetUnsigned64,
    pub(super) reset_status: Option<ResetStatus>,
    // Validate the whole advertised extension, including functions we do not call.
    _is_query: IsQuery,
    _query_counter: Begin,
    _get_signed: GetQuery,
    _get_signed64: GetSigned64,
    _get_integer64: GetInteger64,
}

// Each invocation has a concrete ABI type, never a generic transmute. EGL's
// loader requires a current context. Non-null alone is not extension support.
macro_rules! load {
    ($name:literal, $ty:ty) => {{
        let address = unsafe { get_proc_address($name) };
        if address.is_null() {
            return None;
        }
        unsafe { std::mem::transmute::<*const c_void, $ty>(address) }
    }};
}

impl Extension {
    /// The caller must keep this renderer's EGL context current throughout use.
    pub(super) unsafe fn load(gl: &ffi::Gles2) -> Option<Self> {
        // GetString is valid for EXTENSIONS on GLES 2 and later. No GL error
        // draining: consuming the renderer's error flags would hide its errors.
        let extensions = unsafe { gl.GetString(ffi::EXTENSIONS) };
        if extensions.is_null() {
            return None;
        }
        let extensions = unsafe { CStr::from_ptr(extensions.cast()) }.to_bytes();
        let advertised = |name: &[u8]| extensions.split(u8::is_ascii_whitespace).any(|s| s == name);
        if !advertised(b"GL_EXT_disjoint_timer_query") {
            return None;
        }

        let version = unsafe { gl.GetString(ffi::VERSION) };
        if version.is_null() {
            return None;
        }
        let version = unsafe { CStr::from_ptr(version.cast()) }.to_str().ok()?;
        let version = version
            .strip_prefix("OpenGL ES ")?
            .split_ascii_whitespace()
            .next()?;
        let (major, minor) = version.split_once('.')?;
        let version = (major.parse::<u32>().ok()?, minor.parse::<u32>().ok()?);
        let reset_status = if version >= (3, 2) {
            Some(load!("glGetGraphicsResetStatus", ResetStatus))
        } else if advertised(b"GL_KHR_robustness") {
            Some(load!("glGetGraphicsResetStatusKHR", ResetStatus))
        } else if advertised(b"GL_EXT_robustness") {
            Some(load!("glGetGraphicsResetStatusEXT", ResetStatus))
        } else {
            None
        };
        Some(Self {
            gen_queries: load!("glGenQueriesEXT", Gen),
            delete_queries: load!("glDeleteQueriesEXT", Delete),
            begin_query: load!("glBeginQueryEXT", Begin),
            end_query: load!("glEndQueryEXT", End),
            get_query: load!("glGetQueryivEXT", GetQuery),
            get_unsigned: load!("glGetQueryObjectuivEXT", GetUnsigned),
            get_unsigned64: load!("glGetQueryObjectui64vEXT", GetUnsigned64),
            reset_status,
            _is_query: load!("glIsQueryEXT", IsQuery),
            _query_counter: load!("glQueryCounterEXT", Begin),
            _get_signed: load!("glGetQueryObjectivEXT", GetQuery),
            _get_signed64: load!("glGetQueryObjecti64vEXT", GetSigned64),
            _get_integer64: load!("glGetInteger64vEXT", GetInteger64),
        })
    }
}
