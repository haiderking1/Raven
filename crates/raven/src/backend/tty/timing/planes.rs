use super::Timing;
use crate::backend::tty::render::RenderOutcome;

impl Timing {
    pub fn planes(&mut self, outcome: &RenderOutcome) {
        if !outcome.queued {
            return;
        }
        self.window.primary_scanout_queued += u64::from(outcome.primary_scanout);
        self.window.composition_queued += u64::from(!outcome.primary_scanout);
        self.window.cursor_queued += u64::from(outcome.hardware_cursor);
        self.window.plane_recoveries += u64::from(outcome.recovered);
    }
}
