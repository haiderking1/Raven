use super::{Phase, encoding, model::Selection};
use crate::state::State;
use smithay::reexports::calloop::LoopHandle;
impl State {
    pub(crate) fn install_screenshot(&mut self, handle: LoopHandle<'static, Self>) {
        self.screenshot.handle = Some(handle);
    }
    pub(crate) fn open_screenshot(&mut self) {
        if self.screenshot.active() || self.screenshot.encoding.is_some() {
            return;
        }
        if self.configuration_drag_active()
            || self.dnd_icon.is_some()
            || self.exclusive_keyboard_layer().is_some()
            || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
            || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
        {
            return;
        }
        let Some(output) = &self.output else {
            return;
        };
        let Some(area) = self.space().output_geometry(output) else {
            return;
        };
        let Some(mode) = output.current_mode() else {
            return;
        };
        let size = output.current_transform().transform_size(mode.size);
        if Selection::new(size).is_none() {
            self.screenshot_notice("Screenshot dimensions exceed the capture limit");
            return;
        }
        self.screenshot.area = area;
        self.screenshot.size = size;
        self.screenshot.scale = output.current_scale().fractional_scale();
        self.screenshot.origin = self.pointer_location;
        self.screenshot.generation = self.screenshot.generation.wrapping_add(1);
        self.screenshot.phase = Phase::Requested;
        self.screenshot.pending_selection = Selection::new(size);
        self.screenshot.confirm_pending = false;
        self.screenshot.notice = None;
        self.stop_switcher_repeat();
        self.request_redraw();
    }
    pub(crate) fn screenshot_ready(&mut self, generation: u64) {
        if generation != self.screenshot.generation
            || !matches!(self.screenshot.phase, Phase::Requested)
        {
            return;
        }
        self.screenshot.phase = Phase::Selecting(
            self.screenshot
                .pending_selection
                .take()
                .expect("pending selection"),
        );
        self.screenshot.commit.increment();
        self.cancel_app_switcher();
        if self.screenshot.confirm_pending {
            self.confirm_screenshot();
        }
        self.request_redraw();
    }
    pub(crate) fn confirm_screenshot(&mut self) {
        if matches!(self.screenshot.phase, Phase::Requested) {
            if self
                .screenshot
                .pending_selection
                .as_ref()
                .is_some_and(|s| !s.dragging())
            {
                self.screenshot.confirm_pending = true;
            }
            return;
        }
        let Phase::Selecting(selection) = &self.screenshot.phase else {
            return;
        };
        if selection.dragging() {
            return;
        }
        self.screenshot.phase = Phase::Exporting(selection.rect);
        self.request_redraw();
    }
    pub(crate) fn cancel_screenshot(&mut self) {
        if !self.screenshot.active() {
            return;
        }
        self.screenshot.phase = Phase::Idle;
        self.screenshot.pending_selection = None;
        self.screenshot.confirm_pending = false;
        self.screenshot.generation = self.screenshot.generation.wrapping_add(1);
        self.pointer_location = self.screenshot.origin;
        if let Some(area) = self
            .output
            .as_ref()
            .and_then(|o| self.space().output_geometry(o))
            .filter(|r| r.size.w > 0 && r.size.h > 0)
        {
            self.pointer_location.x = self
                .pointer_location
                .x
                .clamp(area.loc.x as f64, (area.loc.x + area.size.w - 1) as f64);
            self.pointer_location.y = self
                .pointer_location
                .y
                .clamp(area.loc.y as f64, (area.loc.y + area.size.h - 1) as f64);
        }
        self.cancel_app_switcher();
        self.restore_focus();
        self.refresh_tiling_pointer();
        self.request_redraw();
    }
    pub(crate) fn screenshot_pixels(
        &mut self,
        generation: u64,
        image: Result<encoding::Pixels, String>,
    ) {
        if self.screenshot.generation != generation
            || !matches!(self.screenshot.phase, Phase::Exporting(_))
        {
            return;
        }
        self.cancel_screenshot();
        match image.and_then(|image| encoding::start(image, self.loop_signal.clone())) {
            Ok(receiver) => {
                self.screenshot.encoding = Some(receiver);
                self.screenshot_notice("Saving screenshot");
            }
            Err(error) => self.screenshot_notice(&format!("Screenshot failed\n{error}")),
        }
    }
}
