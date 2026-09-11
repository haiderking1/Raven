use super::element::BorderElement;
use smithay::{
    backend::renderer::element::{
        Element, Kind,
        solid::{SolidColorBuffer, SolidColorRenderElement},
    },
    utils::{Physical, Rectangle},
};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct BorderCache {
    pub stripes: [Stripe; 4],
}

#[derive(Default)]
pub(super) struct Stripe {
    buffer: SolidColorBuffer,
    element: Option<BorderElement>,
}

impl Stripe {
    pub fn element(
        &mut self,
        geometry: Rectangle<i32, Physical>,
        color: [f32; 4],
    ) -> BorderElement {
        let unchanged = self
            .element
            .as_ref()
            .is_some_and(|e| e.geometry(1.0.into()) == geometry && e.0.color() == color.into());
        if !unchanged {
            // Physical shared edges were rounded before this call. Use scale 1
            // rather than independently rounding a stripe's logical width.
            self.buffer
                .update((geometry.size.w, geometry.size.h), color);
            self.element = Some(BorderElement(Arc::new(
                SolidColorRenderElement::from_buffer(
                    &self.buffer,
                    geometry.loc,
                    1.0,
                    1.0,
                    Kind::Unspecified,
                ),
            )));
        }
        self.element.as_ref().unwrap().clone()
    }
}
