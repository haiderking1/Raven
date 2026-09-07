use super::super::{device::Device, dmabuf::FeedbackDelivery, presentation::QueuedFeedback};
use super::{RenderOutcome, SceneElement, recovery};
use crate::state::State;
use smithay::backend::{
    drm::compositor::{FrameFlags, PrimaryPlaneElement},
    renderer::element::Element,
};
use std::{error::Error, io};

pub(super) fn render(
    device: &mut Device,
    state: &mut State,
    elements: &[SceneElement],
    delivery: &mut FeedbackDelivery,
    deferred: bool,
) -> Result<RenderOutcome, Box<dyn Error>> {
    let mut rejected = None;
    for attempt in 0..2 {
        let flags = if attempt == 0 {
            device.plane_policy.flags()
        } else {
            FrameFlags::empty()
        };
        if let Some(timer) = &mut device.gpu_time {
            timer.begin(&mut device.renderer)?;
        }
        let rendered_frame = device.compositor.render_frame(
            &mut device.renderer,
            elements,
            [0.055, 0.065, 0.085, 1.0],
            flags,
        );
        // Close the query even when rendering failed, before waiting or queueing.
        let query_end = if let Some(timer) = &mut device.gpu_time {
            timer.end(&mut device.renderer).and_then(|()| {
                if timer.is_enabled() {
                    // finish() flushed the render fence before our end marker.
                    // Submit the marker without waiting for its result.
                    device.renderer.with_context(|gl| unsafe { gl.Flush() })
                } else {
                    Ok(())
                }
            })
        } else {
            Ok(())
        };
        let mut frame = rendered_frame.map_err(|error| recovery::failure(&rejected, error))?;
        query_end.map_err(|error| recovery::failure(&rejected, error))?;
        if frame.needs_sync()
            && let PrimaryPlaneElement::Swapchain(element) = &frame.primary_element
        {
            let boundary = state
                .input_timing
                .as_mut()
                .map(|timing| timing.before_wait());
            let result = element.sync.wait();
            if let (Some(timing), Some(boundary)) = (&mut state.input_timing, boundary) {
                timing.after_wait(boundary, result.is_ok());
            }
            result.map_err(|error| recovery::failure(&rejected, error))?;
        }
        let changed = !frame.is_empty;
        let primary_scanout = matches!(&frame.primary_element, PrimaryPlaneElement::Element(_));
        let used_planes = primary_scanout || frame.cursor_element.is_some();
        let copied_cursor = frame.cursor_element.map(|element| element.id().clone());
        let rendered = std::mem::take(&mut frame.states);
        let queued_feedback = changed.then(QueuedFeedback::default);
        // Release result borrows before queueing or allocating the retry. The
        // complete scene above remains intact, including any cursor candidate.
        drop(frame);
        if let Some(feedback) = &device.feedback {
            delivery.update(
                state,
                &device.output,
                &rendered,
                copied_cursor.as_ref(),
                feedback,
                flags.contains(FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT),
            );
        }
        let Some(queued_feedback) = queued_feedback else {
            if rejected.is_some() {
                return Err(recovery::failure(
                    &rejected,
                    io::Error::other("forced composition repaint produced no frame"),
                ));
            }
            state.update_render_visibility(&device.output, &rendered);
            return Ok(RenderOutcome::default());
        };
        match device.compositor.queue_frame(queued_feedback.clone()) {
            Ok(()) => {
                queued_feedback.set(state.take_presentation_feedback(
                    &device.output,
                    &rendered,
                    copied_cursor.as_ref(),
                ));
                if let Some(timing) = &mut state.input_timing {
                    timing.frame_queued(deferred);
                }
                state.update_render_visibility(&device.output, &rendered);
                return Ok(RenderOutcome {
                    queued: true,
                    feedback: Some(queued_feedback),
                    primary_scanout,
                    hardware_cursor: copied_cursor.is_some(),
                    recovered: rejected.is_some(),
                });
            }
            Err(error)
                if attempt == 0
                    && used_planes
                    && device.drm.is_atomic()
                    && recovery::plane_rejection(&error) =>
            {
                rejected = Some(error.to_string());
                // Do not recover after a VT loss or unplug. Neither reset_buffers
                // nor another render may touch an unavailable device.
                if !device.drm.is_active() {
                    return Err(recovery::failure(
                        &rejected,
                        io::Error::other("DRM device became inactive"),
                    ));
                }
                device
                    .validate_output()
                    .map_err(|error| recovery::failure(&rejected, error))?;
                eprintln!(
                    "raven: plane queue rejected ({error}); retrying once with full composition"
                );
                device.compositor.reset_buffers();
                device.plane_policy.suspend();
                // Feedback remains on the surfaces until an attempt succeeds.
                // Avoid repeating the same rejected plane commit every frame.
            }
            Err(error) => return Err(recovery::failure(&rejected, error)),
        }
    }
    unreachable!("second queue attempt either succeeds or returns its error")
}
