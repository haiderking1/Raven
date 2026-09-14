mod capture;
mod download;
#[cfg(test)]
mod tests;
mod view;
use super::SceneElement;
use crate::{
    desktop::screenshot::{Phase, encoding::Pixels},
    state::State,
};
use smithay::{
    backend::renderer::{
        element::{Id, Kind, memory::MemoryRenderBufferRenderElement},
        gles::{GlesError, GlesRenderer},
    },
    output::Output,
};
pub(super) use view::View;
#[derive(Default)]
pub(super) struct Capture {
    image: Option<capture::Captured>,
    generation: u64,
    view_id: Option<Id>,
    download: Option<download::Download>,
}
impl Capture {
    pub fn append(
        &mut self,
        renderer: &mut GlesRenderer,
        state: &State,
        output: &Output,
        elements: &mut Vec<SceneElement>,
    ) -> Result<(), GlesError> {
        if let Some(image) = &mut self.image {
            image.retire_inputs();
        }
        if !state.screenshot.active() {
            self.image = None;
            self.download = None;
        }
        let Some(area) = state.space().output_geometry(output) else {
            return Ok(());
        };
        let scale = output.current_scale().fractional_scale();
        let control = if state.screenshot.active() {
            None
        } else {
            state.screenshot.notice.as_ref()
        };
        if let Some(buffer) =
            control.filter(|_| !matches!(state.screenshot.phase, Phase::Requested))
        {
            let width = 360.min((area.size.w - 16).max(1));
            let height = 64.min(area.size.h.max(1));
            let location = (
                (area.size.w - width) as f64 / 2.0,
                (area.size.h - height - 20).max(0) as f64,
            );
            let element = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                smithay::utils::Point::from(location).to_physical(scale),
                buffer,
                None,
                Some(smithay::utils::Rectangle::from_size((360.0, 64.0).into())),
                Some((width, height).into()),
                Kind::Unspecified,
            )?;
            elements.push(SceneElement::Memory(element));
        }
        if self.generation == state.screenshot.generation {
            let selection = match &state.screenshot.phase {
                Phase::Selecting(s) => Some(s.outlined.then_some(s.rect)),
                Phase::Exporting(r) => Some(Some(*r)),
                _ => None,
            };
            if let (Some(selection), Some(image)) = (selection, &self.image) {
                elements.push(SceneElement::Screenshot(View {
                    id: self.view_id.get_or_insert_with(Id::new).clone(),
                    commit: state.screenshot.commit,
                    texture: image.image.clone(),
                    size: state.screenshot.size,
                    selection,
                    scale,
                }));
            }
        }
        Ok(())
    }
    pub fn after_frame(
        &mut self,
        renderer: &mut GlesRenderer,
        state: &mut State,
        elements: &mut Vec<SceneElement>,
    ) {
        let result = if matches!(state.screenshot.phase, Phase::Requested) {
            let clear = if state.fullscreen_window().is_some() {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [0.055, 0.065, 0.085, 1.0]
            };
            capture::capture(
                renderer,
                state.screenshot.size,
                state.screenshot.scale,
                std::mem::take(elements),
                clear,
            )
            .map(|image| {
                self.image = Some(image);
                self.generation = state.screenshot.generation;
                self.view_id = Some(Id::new());
                state.screenshot_ready(self.generation);
            })
        } else if let Phase::Exporting(rect) = state.screenshot.phase {
            if self.download.is_none() && self.generation == state.screenshot.generation {
                match self.image.as_mut() {
                    Some(image) => download::Download::start(
                        renderer,
                        &mut image.image,
                        rect,
                        self.generation,
                        state.loop_signal.clone(),
                    )
                    .map(|d| self.download = Some(d)),
                    None => Err("frozen screenshot image is unavailable".into()),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        if let Err(error) = result {
            state.cancel_screenshot();
            self.image = None;
            self.download = None;
            state.screenshot_notice(&format!("Screenshot failed\n{error}"));
        }
    }
    pub fn poll(&mut self, renderer: &mut GlesRenderer) -> Option<(u64, Result<Pixels, String>)> {
        if let Some(image) = &mut self.image {
            image.retire_inputs();
        }
        if !self.download.as_ref().is_some_and(|d| d.ready()) {
            return None;
        }
        let download = self.download.take()?;
        Some((download.generation, download.finish(renderer)))
    }
}
