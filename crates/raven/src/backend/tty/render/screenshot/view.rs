use smithay::{
    backend::renderer::{
        Texture,
        element::{Element, Id, RenderElement},
        gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture},
        utils::{CommitCounter, OpaqueRegions},
    },
    utils::{Buffer, Physical, Rectangle, Scale, Size, Transform},
};
#[derive(Clone, Debug)]
pub(crate) struct View {
    pub id: Id,
    pub commit: CommitCounter,
    pub texture: GlesTexture,
    pub size: Size<i32, Physical>,
    pub selection: Option<Rectangle<i32, Physical>>,
    pub scale: f64,
}
impl Element for View {
    fn id(&self) -> &Id {
        &self.id
    }
    fn current_commit(&self) -> CommitCounter {
        self.commit
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size(self.texture.size()).to_f64()
    }
    fn geometry(&self, _: Scale<f64>) -> Rectangle<i32, Physical> {
        Rectangle::from_size(self.size)
    }
    fn opaque_regions(&self, _: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::from_slice(&[Rectangle::from_size(self.size)])
    }
}
impl RenderElement<GlesRenderer> for View {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        _: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        frame.render_texture_from_to(
            &self.texture,
            src,
            dst,
            damage,
            &[],
            Transform::Normal,
            1.0,
            None,
            &[],
        )?;
        let r = self.selection.unwrap_or_default();
        let w = self.size.w;
        let h = self.size.h;
        let mut solid =
            |rect: Rectangle<i32, Physical>, color: [f32; 4]| -> Result<(), GlesError> {
                if rect.is_empty() {
                    return Ok(());
                }
                let clips: Vec<_> = damage
                    .iter()
                    .filter_map(|d| d.intersection(rect))
                    .map(|mut d| {
                        d.loc -= rect.loc;
                        d
                    })
                    .collect();
                frame.draw_solid(rect, &clips, color.into())
            };
        for rect in [
            Rectangle::new((0, 0).into(), (w, r.loc.y).into()),
            Rectangle::new(
                (0, r.loc.y + r.size.h).into(),
                (w, h - r.loc.y - r.size.h).into(),
            ),
            Rectangle::new((0, r.loc.y).into(), (r.loc.x, r.size.h).into()),
            Rectangle::new(
                (r.loc.x + r.size.w, r.loc.y).into(),
                (w - r.loc.x - r.size.w, r.size.h).into(),
            ),
        ] {
            solid(rect, [0.0, 0.0, 0.0, 0.5])?;
        }
        if self.selection.is_none() || r == Rectangle::from_size(self.size) {
            return Ok(());
        }
        // Keep the outline outside the selected pixels. Output-edge clipping
        // naturally hides borders which fall beyond the screen.
        let t = (self.scale * 2.0).round().max(1.0) as i32;
        for rect in [
            Rectangle::new(
                (r.loc.x - t, r.loc.y - t).into(),
                (r.size.w + 2 * t, t).into(),
            ),
            Rectangle::new(
                (r.loc.x - t, r.loc.y + r.size.h).into(),
                (r.size.w + 2 * t, t).into(),
            ),
            Rectangle::new((r.loc.x - t, r.loc.y).into(), (t, r.size.h).into()),
            Rectangle::new((r.loc.x + r.size.w, r.loc.y).into(), (t, r.size.h).into()),
        ] {
            solid(rect, [1.0, 1.0, 1.0, 1.0])?;
        }
        Ok(())
    }
}
