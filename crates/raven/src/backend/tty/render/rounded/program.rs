use smithay::backend::renderer::gles::{
    GlesError, GlesPixelProgram, GlesRenderer, GlesTexProgram, UniformName, UniformType,
};
use std::rc::Rc;
#[derive(Debug)]
pub(super) struct Programs {
    pub texture: GlesTexProgram,
    pub ring: GlesPixelProgram,
}
pub(in crate::backend::tty::render) fn source(shader: &str) -> String {
    shader.replace("//_CORNERS", include_str!("shaders/mask.glsl"))
}
pub(in crate::backend::tty::render) fn uniforms() -> [UniformName<'static>; 2] {
    [
        UniformName::new("corner_bounds", UniformType::_4f),
        UniformName::new("corner_radius", UniformType::_1f),
    ]
}
impl Programs {
    pub fn get(renderer: &mut GlesRenderer) -> Result<Rc<Self>, GlesError> {
        if let Some(program) = renderer.egl_context().user_data().get::<Rc<Self>>() {
            return Ok(program.clone());
        }
        let texture = renderer.compile_custom_texture_shader(
            &source(include_str!("shaders/surface.frag")),
            &uniforms(),
        )?;
        let mut names = uniforms().to_vec();
        names.extend([
            UniformName::new("ring_color", UniformType::_4f),
            UniformName::new("inner_bounds", UniformType::_4f),
            UniformName::new("inner_radius", UniformType::_1f),
        ]);
        let ring = renderer
            .compile_custom_pixel_shader(&source(include_str!("shaders/ring.frag")), &names)?;
        let programs = Rc::new(Self { texture, ring });
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| programs.clone());
        Ok(programs)
    }
}
