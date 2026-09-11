use smithay::{
    backend::renderer::{
        Renderer,
        element::{Element, Id, RenderElement, solid::SolidColorRenderElement},
        utils::{CommitCounter, OpaqueRegions},
    },
    utils::{Buffer, Physical, Rectangle, Scale},
};
use std::sync::Arc;

/// Cloning a Smithay solid element copies its opaque-region Vec. Share the
/// immutable snapshot instead; ordinary scene rebuilds allocate nothing here.
#[derive(Clone, Debug)]
pub(crate) struct BorderElement(pub(super) Arc<SolidColorRenderElement>);

impl Element for BorderElement {
    fn id(&self) -> &Id {
        self.0.id()
    }
    fn current_commit(&self) -> CommitCounter {
        self.0.current_commit()
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        self.0.src()
    }
    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.0.geometry(scale)
    }
    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.0.opaque_regions(scale)
    }
    fn alpha(&self) -> f32 {
        self.0.alpha()
    }
}

impl<R: Renderer> RenderElement<R> for BorderElement {
    fn draw(
        &self,
        frame: &mut R::Frame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque: &[Rectangle<i32, Physical>],
    ) -> Result<(), R::Error> {
        <SolidColorRenderElement as RenderElement<R>>::draw(
            &self.0, frame, src, dst, damage, opaque,
        )
    }
}
