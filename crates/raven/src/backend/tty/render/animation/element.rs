use smithay::{
    backend::renderer::{
        Renderer,
        element::{Element, Id, Kind, RenderElement},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
    },
    utils::{Buffer, Physical, Rectangle, Scale, Transform},
};

/// Keep live wl_surface identity and participation, but forbid direct scanout.
/// The synthetic commit covers opacity changes even when rounded geometry stays
/// unchanged. Full local damage also includes every newly committed live pixel.
#[derive(Debug)]
pub(in crate::backend::tty::render) struct Animated<E> {
    pub element: E,
    pub commit: CommitCounter,
    pub opaque: Vec<Rectangle<i32, Physical>>,
}

impl<E: Element> Element for Animated<E> {
    fn id(&self) -> &Id {
        self.element.id()
    }
    fn current_commit(&self) -> CommitCounter {
        self.commit
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        self.element.src()
    }
    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.element.geometry(scale)
    }
    fn transform(&self) -> Transform {
        self.element.transform()
    }
    fn alpha(&self) -> f32 {
        self.element.alpha()
    }
    fn kind(&self) -> Kind {
        self.element.kind()
    }
    fn opaque_regions(&self, _: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::from_slice(&self.opaque)
    }
    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit == Some(self.commit) {
            DamageSet::default()
        } else {
            DamageSet::from_slice(&[Rectangle::from_size(self.geometry(scale).size)])
        }
    }
}

impl<R: Renderer, E: RenderElement<R>> RenderElement<R> for Animated<E> {
    fn draw(
        &self,
        frame: &mut R::Frame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque: &[Rectangle<i32, Physical>],
    ) -> Result<(), R::Error> {
        self.element.draw(frame, src, dst, damage, opaque)
    }
    // No underlying_storage: these transformed/faded draws are composited. In
    // particular, retaining a wl_buffer must not report ZERO_COPY presentation.
}
