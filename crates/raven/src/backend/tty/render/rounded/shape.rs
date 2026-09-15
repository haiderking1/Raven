use smithay::{
    backend::renderer::{
        gles::{GlesError, GlesFrame, Uniform, ffi},
        utils::OpaqueRegions,
    },
    utils::{Physical, Rectangle},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Shape {
    pub bounds: Rectangle<i32, Physical>,
    pub radius: f32,
}
impl Shape {
    pub fn new(bounds: Rectangle<i32, Physical>, radius: f32) -> Self {
        Self {
            bounds,
            radius: radius
                .min(bounds.size.w.min(bounds.size.h) as f32 / 2.0)
                .max(0.0),
        }
    }
    pub fn opaque(
        &self,
        regions: OpaqueRegions<i32, Physical>,
        geometry: Rectangle<i32, Physical>,
    ) -> OpaqueRegions<i32, Physical> {
        let r = self.radius.ceil() as i32;
        let b = self.bounds;
        let safe = [
            Rectangle::new(
                (b.loc.x + r, b.loc.y).into(),
                ((b.size.w - 2 * r).max(0), b.size.h).into(),
            ),
            Rectangle::new(
                (b.loc.x, b.loc.y + r).into(),
                (b.size.w, (b.size.h - 2 * r).max(0)).into(),
            ),
        ];
        regions
            .into_iter()
            .flat_map(|mut region| {
                region.loc += geometry.loc;
                safe.into_iter()
                    .filter_map(move |safe| region.intersection(safe))
                    .map(|mut r| {
                        r.loc -= geometry.loc;
                        r
                    })
            })
            .collect()
    }
    pub fn uniforms(
        &self,
        frame: &mut GlesFrame<'_, '_>,
    ) -> Result<Vec<Uniform<'static>>, GlesError> {
        self.uniforms_named(frame, "corner_bounds", "corner_radius")
    }
    pub fn uniforms_named(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        bounds_name: &'static str,
        radius_name: &'static str,
    ) -> Result<Vec<Uniform<'static>>, GlesError> {
        let p = *frame.projection();
        let viewport = frame.with_context(|gl| {
            let mut v = [0i32; 4];
            unsafe {
                gl.GetIntegerv(ffi::VIEWPORT, v.as_mut_ptr());
            }
            v
        })?;
        let b = self.bounds;
        let mut lo = [f32::INFINITY; 2];
        let mut hi = [f32::NEG_INFINITY; 2];
        for (x, y) in [
            (b.loc.x, b.loc.y),
            (b.loc.x + b.size.w, b.loc.y),
            (b.loc.x, b.loc.y + b.size.h),
            (b.loc.x + b.size.w, b.loc.y + b.size.h),
        ] {
            let x = x as f32;
            let y = y as f32;
            let q = [
                (p[0] * x + p[3] * y + p[6] + 1.0) * viewport[2] as f32 / 2.0 + viewport[0] as f32,
                (p[1] * x + p[4] * y + p[7] + 1.0) * viewport[3] as f32 / 2.0 + viewport[1] as f32,
            ];
            for i in 0..2 {
                lo[i] = lo[i].min(q[i]);
                hi[i] = hi[i].max(q[i]);
            }
        }
        Ok(vec![
            Uniform::new(bounds_name, [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]]),
            Uniform::new(radius_name, self.radius),
        ])
    }
}
