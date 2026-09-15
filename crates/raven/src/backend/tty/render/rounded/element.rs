use super::super::SceneElement;
use super::{program::Programs, shape::Shape};
use smithay::{
    backend::renderer::{
        element::{Element, Id, Kind, RenderElement},
        gles::{GlesError, GlesFrame, GlesRenderer, Uniform},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
    },
    utils::{Buffer, Physical, Rectangle, Scale, Transform},
};
use std::rc::Rc;
pub(crate) struct Rounded {
    pub element: Box<SceneElement>,
    pub shape: Shape,
    pub commit: CommitCounter,
    pub previous: Option<(CommitCounter, CommitCounter)>,
    pub scale: f64,
    pub(super) programs: Rc<Programs>,
}
impl Element for Rounded {
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
    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.shape
            .opaque(self.element.opaque_regions(scale), self.geometry(scale))
    }
    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit == Some(self.commit) {
            DamageSet::default()
        } else if let Some((_, source)) = self
            .previous
            .filter(|(previous, _)| commit == Some(*previous))
        {
            self.element.damage_since(scale, Some(source))
        } else {
            DamageSet::from_slice(&[Rectangle::from_size(self.geometry(scale).size)])
        }
    }
}
impl RenderElement<GlesRenderer> for Rounded {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let mut shape = self.shape;
        shape.bounds.loc += dst.loc - self.geometry(self.scale.into()).loc;
        let uniforms = shape.uniforms(frame)?;
        frame.override_default_tex_program(self.programs.texture.clone(), uniforms);
        let result = self.element.draw(frame, src, dst, damage, opaque);
        frame.clear_tex_program_override();
        result
    }
    // Keep live element identity, but do not advertise a masked buffer for scanout.
}
#[derive(Clone, Debug)]
pub(crate) struct Ring {
    pub id: Id,
    pub commit: CommitCounter,
    pub shape: Shape,
    pub color: [f32; 4],
    pub inner: Shape,
    pub(super) programs: Rc<Programs>,
}
impl Element for Ring {
    fn id(&self) -> &Id {
        &self.id
    }
    fn current_commit(&self) -> CommitCounter {
        self.commit
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size((self.shape.bounds.size.w, self.shape.bounds.size.h).into()).to_f64()
    }
    fn geometry(&self, _: Scale<f64>) -> Rectangle<i32, Physical> {
        self.shape.bounds
    }
    fn alpha(&self) -> f32 {
        self.color[3]
    }
}
impl RenderElement<GlesRenderer> for Ring {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        _: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let mut shape = self.shape;
        shape.bounds = dst;
        let mut inner = self.inner;
        inner.bounds.loc += dst.loc - self.shape.bounds.loc;
        let mut uniforms = shape.uniforms(frame)?;
        uniforms.extend(inner.uniforms_named(frame, "inner_bounds", "inner_radius")?);
        uniforms.push(Uniform::new("ring_color", self.color));
        frame.render_pixel_shader_to(
            &self.programs.ring,
            src,
            dst,
            (dst.size.w, dst.size.h).into(),
            Some(damage),
            1.0,
            &uniforms,
        )
    }
}

impl std::fmt::Debug for Rounded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rounded")
            .field("shape", &self.shape)
            .finish_non_exhaustive()
    }
}
