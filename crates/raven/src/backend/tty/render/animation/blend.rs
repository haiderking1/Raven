use super::snapshot::Snapshot;
use smithay::{
    backend::renderer::{
        Texture,
        element::{Element, Id, RenderElement},
        gles::{
            GlesError, GlesFrame, GlesRenderer, GlesTexProgram, Uniform, UniformName, UniformType,
            ffi,
        },
        utils::{CommitCounter, OpaqueRegions},
    },
    utils::{Buffer, Physical, Rectangle, Scale, Transform},
};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(in crate::backend::tty::render) struct Blend {
    pub corners: Option<super::super::rounded::shape::Shape>,
    pub id: Id,
    pub old: Rc<Snapshot>,
    pub current: Rc<Snapshot>,
    pub bounds: Rectangle<i32, Physical>,
    pub progress: f32,
    pub commit: CommitCounter,
    pub program: Rc<GlesTexProgram>,
    /// Output-coordinate regions where each live surface contributes to current.
    pub live: Rc<[(Id, Vec<Rectangle<i32, Physical>>)]>,
    pub opaque: Rc<[Rectangle<i32, Physical>]>,
}

pub(super) fn compile(
    renderer: &mut GlesRenderer,
) -> Result<Rc<GlesTexProgram>, Box<dyn std::error::Error>> {
    Ok(Rc::new(renderer.compile_custom_texture_shader(
        &super::super::rounded::program::source(include_str!("shaders/resize.frag")),
        &[
            UniformName::new("previous_image", UniformType::_1i),
            UniformName::new("progress", UniformType::_1f),
            UniformName::new("corner_bounds", UniformType::_4f),
            UniformName::new("corner_radius", UniformType::_1f),
        ],
    )?))
}

impl Element for Blend {
    fn id(&self) -> &Id {
        &self.id
    }
    fn current_commit(&self) -> CommitCounter {
        self.commit
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size(self.current.texture.size()).to_f64()
    }
    fn geometry(&self, _: Scale<f64>) -> Rectangle<i32, Physical> {
        self.bounds
    }
    fn opaque_regions(&self, _: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        let regions = OpaqueRegions::from_slice(&self.opaque);
        if let Some(shape) = self.corners {
            shape.opaque(regions, self.bounds)
        } else {
            regions
        }
    }
}

impl RenderElement<GlesRenderer> for Blend {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let mut shape = self
            .corners
            .unwrap_or(super::super::rounded::shape::Shape::new(self.bounds, 0.0));
        shape.bounds.loc += dst.loc - self.bounds.loc;
        let mut uniforms = shape.uniforms(frame)?;
        uniforms.extend([
            Uniform::new("previous_image", 1i32),
            Uniform::new("progress", self.progress),
        ]);
        let previous = frame.with_context(|gl| unsafe {
            // Both images are compositor-owned immutable 2D textures produced on
            // this context. No external client texture or buffer name is bound.
            let mut active = 0;
            let mut binding = 0;
            gl.GetIntegerv(ffi::ACTIVE_TEXTURE, &mut active);
            gl.ActiveTexture(ffi::TEXTURE1);
            gl.GetIntegerv(ffi::TEXTURE_BINDING_2D, &mut binding);
            gl.BindTexture(ffi::TEXTURE_2D, self.old.texture.tex_id());
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MIN_FILTER, ffi::LINEAR as i32);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MAG_FILTER, ffi::LINEAR as i32);
            gl.TexParameteri(
                ffi::TEXTURE_2D,
                ffi::TEXTURE_WRAP_S,
                ffi::CLAMP_TO_EDGE as i32,
            );
            gl.TexParameteri(
                ffi::TEXTURE_2D,
                ffi::TEXTURE_WRAP_T,
                ffi::CLAMP_TO_EDGE as i32,
            );
            gl.ActiveTexture(ffi::TEXTURE0);
            (active, binding)
        })?;
        let result = frame.render_texture_from_to(
            &self.current.texture,
            src,
            dst,
            damage,
            opaque,
            Transform::Normal,
            1.0,
            Some(&self.program),
            &uniforms,
        );
        let restored = frame.with_context(|gl| unsafe {
            gl.ActiveTexture(ffi::TEXTURE1);
            gl.BindTexture(ffi::TEXTURE_2D, previous.1 as u32);
            gl.ActiveTexture(previous.0 as u32);
        });
        result.and(restored)
    }
    // The group and all of its live contributions are composited, never scanout.
}
