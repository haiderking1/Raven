use super::SceneElement;
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{Kind, memory::MemoryRenderBufferRenderElement},
        gles::{GlesError, GlesRenderer},
    },
    output::Output,
};
#[derive(Default)]
pub(super) struct Overlay;
impl Overlay {
    pub fn append(
        &mut self,
        renderer: &mut GlesRenderer,
        state: &State,
        output: &Output,
        elements: &mut Vec<SceneElement>,
    ) -> Result<(), GlesError> {
        let Some(art) = &state.switcher.artwork.frame else {
            return Ok(());
        };
        let Some(area) = state.space().output_geometry(output) else {
            return Ok(());
        };
        let scale = output.current_scale().fractional_scale();
        let location = art.layout.rect.loc - area.loc;
        let memory = MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            location.to_f64().to_physical(scale),
            &art.buffer,
            None,
            None,
            Some(art.layout.rect.size),
            Kind::Unspecified,
        )?;
        elements.push(SceneElement::Memory(memory));
        Ok(())
    }
}
