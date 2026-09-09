use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle},
    wayland::compositor::{
        BufferAssignment, RectangleKind, RegionAttributes, SurfaceAttributes, with_states,
    },
};

/// Surface coordinates, including viewport destination size and the current input region.
pub(super) struct Area {
    bounds: Rectangle<i32, Logical>,
    input: Option<RegionAttributes>,
    explicit: Option<RegionAttributes>,
}

impl Area {
    pub fn new(surface: &WlSurface, explicit: Option<RegionAttributes>) -> Option<Self> {
        let (removed, input) = with_states(surface, |states| {
            let mut attrs = states.cached_state.get::<SurfaceAttributes>();
            let attrs = attrs.current();
            (
                matches!(attrs.buffer, Some(BufferAssignment::Removed)),
                attrs.input_region.clone(),
            )
        });
        if removed {
            return None;
        }
        let size = with_renderer_surface_state(surface, |state| state.surface_size()).flatten()?;
        Some(Self {
            bounds: Rectangle::from_size(size),
            input,
            explicit,
        })
    }

    pub fn contains(&self, point: Point<f64, Logical>) -> bool {
        point.x.is_finite()
            && point.y.is_finite()
            && self.bounds.to_f64().contains(point)
            && self
                .input
                .as_ref()
                .is_none_or(|r| contains_region(r, point))
            && self
                .explicit
                .as_ref()
                .is_none_or(|r| contains_region(r, point))
    }

    /// Stop at the first excluded interval, including holes and disconnected islands.
    /// Splitting at rectangle edges avoids sampling past a narrow excluded region.
    pub fn confine(
        &self,
        from: Point<f64, Logical>,
        to: Point<f64, Logical>,
    ) -> Point<f64, Logical> {
        if !self.contains(from) {
            return from;
        }
        let delta = to - from;
        let mut cuts = vec![0.0, 1.0];
        for rect in std::iter::once(&self.bounds).chain(
            self.input
                .iter()
                .chain(self.explicit.iter())
                .flat_map(|r| r.rects.iter().map(|(_, r)| r)),
        ) {
            for (start, distance, low, high) in [
                (
                    from.x,
                    delta.x,
                    f64::from(rect.loc.x),
                    f64::from(rect.loc.x) + f64::from(rect.size.w),
                ),
                (
                    from.y,
                    delta.y,
                    f64::from(rect.loc.y),
                    f64::from(rect.loc.y) + f64::from(rect.size.h),
                ),
            ] {
                if distance == 0.0 {
                    continue;
                }
                for edge in [low, high] {
                    let t = (edge - start) / distance;
                    if t > 0.0 && t < 1.0 {
                        cuts.push(t);
                    }
                }
            }
        }
        cuts.sort_by(f64::total_cmp);
        for pair in cuts.windows(2) {
            let midpoint = along(from, delta, (pair[0] + pair[1]) / 2.0);
            if !self.contains(midpoint) {
                return self.inside_edge(from, delta, pair[0]);
            }
        }
        if self.contains(to) {
            to
        } else {
            self.inside_edge(from, delta, 1.0)
        }
    }

    fn inside_edge(
        &self,
        from: Point<f64, Logical>,
        delta: Point<f64, Logical>,
        t: f64,
    ) -> Point<f64, Logical> {
        let edge = along(from, delta, t);
        if self.contains(edge) {
            return edge;
        }
        // Stay one Wayland fixed-point quantum inside an exclusive right/bottom edge.
        let back = (1.0 / 256.0) / delta.x.abs().max(delta.y.abs()).max(1.0);
        let point = along(from, delta, (t - back).max(0.0));
        if self.contains(point) { point } else { from }
    }
}

fn along(from: Point<f64, Logical>, delta: Point<f64, Logical>, t: f64) -> Point<f64, Logical> {
    (from.x + delta.x * t, from.y + delta.y * t).into()
}

fn contains_region(region: &RegionAttributes, point: Point<f64, Logical>) -> bool {
    region.rects.iter().fold(false, |inside, (kind, rect)| {
        if rect.to_f64().contains(point) {
            matches!(kind, RectangleKind::Add)
        } else {
            inside
        }
    })
}
