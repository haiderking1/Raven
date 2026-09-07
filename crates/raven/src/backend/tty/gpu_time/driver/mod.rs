//! Current-context GL access. Pool policy is independent of the FFI.

mod load;

pub(super) use load::Extension;
use load::*;
use smithay::backend::renderer::gles::ffi;

pub(super) trait Queries {
    fn reset_detected(&self) -> bool;
    fn disjoint(&self) -> bool;
    fn counter_bits(&self) -> i32;
    fn current(&self) -> u32;
    fn generate(&self, ids: &mut [u32]);
    fn delete(&self, ids: &[u32]);
    fn begin(&self, id: u32);
    fn end(&self);
    fn available(&self, id: u32) -> bool;
    fn result(&self, id: u32) -> u64;
}

pub(super) struct Current<'a> {
    gl: &'a ffi::Gles2,
    extension: &'a Extension,
}

impl<'a> Current<'a> {
    /// Both arguments must belong to the current EGL context. Do not retain
    /// this value across rendering, context switches, or with_context calls.
    pub(super) unsafe fn new(gl: &'a ffi::Gles2, extension: &'a Extension) -> Self {
        Self { gl, extension }
    }
}

impl Extension {
    /// Must be called inside the owning renderer's with_context closure.
    pub(super) unsafe fn for_context(gl: &ffi::Gles2) -> Option<Self> {
        unsafe { Self::load(gl) }
    }
}

impl Queries for Current<'_> {
    fn reset_detected(&self) -> bool {
        self.extension
            .reset_status
            .is_some_and(|status| unsafe { status() != ffi::NO_ERROR })
    }

    fn disjoint(&self) -> bool {
        let mut value = 0;
        unsafe { self.gl.GetIntegerv(DISJOINT, &mut value) };
        value != 0
    }

    fn counter_bits(&self) -> i32 {
        let mut bits = 0;
        unsafe { (self.extension.get_query)(TIME_ELAPSED, COUNTER_BITS, &mut bits) };
        bits
    }

    fn current(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.extension.get_query)(TIME_ELAPSED, CURRENT_QUERY, &mut id) };
        id as u32
    }

    fn generate(&self, ids: &mut [u32]) {
        unsafe { (self.extension.gen_queries)(ids.len() as i32, ids.as_mut_ptr()) };
    }

    fn delete(&self, ids: &[u32]) {
        unsafe { (self.extension.delete_queries)(ids.len() as i32, ids.as_ptr()) };
    }

    fn begin(&self, id: u32) {
        unsafe { (self.extension.begin_query)(TIME_ELAPSED, id) };
    }

    fn end(&self) {
        unsafe { (self.extension.end_query)(TIME_ELAPSED) };
    }

    fn available(&self, id: u32) -> bool {
        let mut available = 0;
        unsafe { (self.extension.get_unsigned)(id, AVAILABLE, &mut available) };
        available != 0
    }

    fn result(&self, id: u32) -> u64 {
        let mut nanoseconds = 0;
        unsafe { (self.extension.get_unsigned64)(id, RESULT, &mut nanoseconds) };
        nanoseconds
    }
}
