//! Nonblocking GL elapsed intervals for compositor-owned DRM render jobs.
//!
//! The adaptive and deadline pipelines use this collector. See README.md for
//! the integration sequence and the context-lifetime contract.
//! Durations are GPU elapsed intervals, not CPU-clock timestamps, isolated
//! shader execution times, or estimates of when pixels reach the display.

mod context;
mod driver;
mod pool;
#[cfg(test)]
mod tests;

use context::Lease;
use driver::{Current, Extension};
use pool::Pool;
use smithay::backend::renderer::gles::{GlesError, GlesRenderer};
use std::{marker::PhantomData, rc::Rc, time::Duration};

struct Collector {
    lease: Lease,
    extension: Extension,
    pool: Pool,
}

/// A four-query collector tied to one live EGL context.
///
/// Keep the original renderer/context alive until explicit destroy succeeds.
/// Use one collector and one disjoint-flag reader per context. Do not switch
/// renderers, use shared contexts as substitutes, or recycle a context beneath
/// this value. No method waits for a query result or flushes the GL queue.
///
/// Dropping this value performs no GL calls. Without explicit cleanup, query
/// objects remain until context destruction, and an open query stays open.
/// The type is deliberately neither Send nor Sync.
pub struct GpuTime {
    collector: Option<Collector>,
    _context_thread: PhantomData<Rc<()>>,
}

impl GpuTime {
    /// Load the advertised EXT entry points and validate the elapsed counter.
    ///
    /// Missing support, missing entry points, unusable counters, an occupied
    /// query target, or a second collector yield an inert value. EGL context
    /// activation errors propagate unchanged from Smithay.
    pub fn new(renderer: &mut GlesRenderer) -> Result<Self, GlesError> {
        let collector = if let Some(lease) = Lease::acquire(renderer) {
            renderer.with_context(|gl| {
                // SAFETY: with_context has made this renderer's EGL context
                // current. The table is used only with this same live context.
                let extension = unsafe { Extension::for_context(gl) }?;
                let current = unsafe { Current::new(gl, &extension) };
                let pool = Pool::new(&current)?;
                Some(Collector {
                    lease,
                    extension,
                    pool,
                })
            })?
        } else {
            None
        };
        Ok(Self {
            collector,
            _context_thread: PhantomData,
        })
    }

    /// Whether timing remains usable, not whether the next begin has a slot.
    /// A faulted collector requires destruction and recreation, not reset.
    pub fn is_enabled(&self) -> bool {
        self.collector
            .as_ref()
            .is_some_and(|collector| collector.pool.enabled())
    }

    /// Begin immediately before this renderer submits the actual DRM GL job.
    ///
    /// Pool exhaustion or a foreign active query skips timing without affecting
    /// rendering. Always pair this with end, even when no measurement starts.
    /// Nested brackets are balanced but their outer sample is discarded.
    pub fn begin(&mut self, renderer: &mut GlesRenderer) -> Result<(), GlesError> {
        self.in_context(renderer, |pool, gl| pool.begin(gl))
            .map(|_| ())
    }

    /// End after the GL frame has finished submitting, including error paths.
    /// Do this before fence waits, KMS queueing, or unrelated renderer work.
    /// An unmatched end is inert and never ends another owner's query.
    pub fn end(&mut self, renderer: &mut GlesRenderer) -> Result<(), GlesError> {
        self.in_context(renderer, |pool, gl| pool.end(gl))
            .map(|_| ())
    }

    /// Poll at most four pending queries once each and return their valid max.
    ///
    /// None means no usable observation, not zero GPU cost. Samples are consumed
    /// once. No availability loop, wait, finish, or flush is issued here.
    pub fn sample(&mut self, renderer: &mut GlesRenderer) -> Result<Option<Duration>, GlesError> {
        self.in_context(renderer, |pool, gl| pool.sample(gl))
            .map(Option::flatten)
    }

    /// Discard all intervals and close an owned open bracket.
    ///
    /// Call after a failed/skipped render, suspend, or scheduling discontinuity.
    /// Invalid pending queries remain reserved until available. Reset does not
    /// revive a faulted/lost context or allocate another pool.
    pub fn reset(&mut self, renderer: &mut GlesRenderer) -> Result<(), GlesError> {
        self.in_context(renderer, |pool, gl| pool.reset(gl))
            .map(|_| ())
    }

    /// Delete owned queries while the original renderer/context is alive.
    ///
    /// Idempotent after success. An activation error retains cleanup ownership
    /// so the caller can retry with the original context. A detected graphics
    /// reset abandons names to the dying context instead of reusing them.
    /// A mismatched renderer disables timing but cannot perform cleanup.
    pub fn destroy(&mut self, renderer: &mut GlesRenderer) -> Result<(), GlesError> {
        if self
            .in_context(renderer, |pool, gl| pool.destroy(gl))?
            .is_some()
        {
            self.collector = None;
        }
        Ok(())
    }

    fn in_context<R>(
        &mut self,
        renderer: &mut GlesRenderer,
        operation: impl FnOnce(&mut Pool, &Current<'_>) -> R,
    ) -> Result<Option<R>, GlesError> {
        let Some(collector) = &mut self.collector else {
            return Ok(None);
        };
        if !collector.lease.matches(renderer) {
            // Query objects are context-local, even in an EGL share group.
            // Keep the original lease/names for cleanup; touch no foreign GL.
            collector.pool.fault();
            return Ok(None);
        }
        let result = renderer.with_context(|gl| {
            // SAFETY: the owning live context was matched and made current.
            // Current never escapes this closure or crosses renderer work.
            let current = unsafe { Current::new(gl, &collector.extension) };
            operation(&mut collector.pool, &current)
        });
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                // The failure may be transient, so retain names for explicit
                // cleanup. Never deliver intervals across an activation error.
                collector.pool.fault();
                Err(error)
            }
        }
    }
}
