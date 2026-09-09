//! Capture request state before wl_surface.commit, then apply it with its surface transaction.
use super::*;
use compositor::Cacheable;
use wayland_server::backend::ObjectId;

#[derive(Debug)]
struct Update {
    id: ObjectId,
    region: Option<RegionAttributes>,
    hint: Option<Point<f64, Logical>>,
}

#[derive(Debug, Default)]
struct Committed(Vec<Update>);

impl Cacheable for Committed {
    fn commit(&mut self, _dh: &DisplayHandle) -> Self {
        std::mem::take(self)
    }
    fn merge_into(self, into: &mut Self, _dh: &DisplayHandle) {
        for update in self.0 {
            if let Some(previous) = into.0.iter_mut().find(|previous| previous.id == update.id) {
                previous.region = update.region;
                if update.hint.is_some() {
                    previous.hint = update.hint;
                }
            } else {
                into.0.push(update);
            }
        }
    }
}

pub(super) fn snapshot<D: PointerConstraintsHandler + 'static>(
    _state: &mut D,
    _dh: &DisplayHandle,
    surface: &WlSurface,
) {
    let updates = with_constraint_data::<D, _, _>(surface, |data| {
        data.unwrap()
            .constraints
            .values_mut()
            .map(|constraint| match constraint {
                PointerConstraint::Confined(c) => Update {
                    id: c.handle.id(),
                    region: c.pending_region.clone(),
                    hint: None,
                },
                PointerConstraint::Locked(c) => Update {
                    id: c.handle.id(),
                    region: c.pending_region.clone(),
                    hint: c.pending_cursor_position_hint.take(),
                },
            })
            .collect()
    });
    compositor::with_states(surface, |states| {
        states.cached_state.get::<Committed>().pending().0 = updates;
    });
}

pub(super) fn apply<D: PointerConstraintsHandler + 'static>(
    surface: &WlSurface,
) -> Vec<(PointerHandle<D>, Option<Point<f64, Logical>>)> {
    let updates = compositor::with_states(surface, |states| {
        std::mem::take(&mut states.cached_state.get::<Committed>().current().0)
    });
    with_constraint_data::<D, _, _>(surface, |data| {
        data.unwrap()
            .constraints
            .iter_mut()
            .filter_map(|(pointer, constraint)| {
                let (id, region, hint) = match constraint {
                    PointerConstraint::Confined(c) => (c.handle.id(), &mut c.region, None),
                    PointerConstraint::Locked(c) => {
                        (c.handle.id(), &mut c.region, Some(&mut c.cursor_position_hint))
                    }
                };
                let update = updates.iter().find(|update| update.id == id)?;
                region.clone_from(&update.region);
                if let Some(hint) = hint {
                    if update.hint.is_some() {
                        *hint = update.hint;
                    }
                }
                Some((pointer.clone(), update.hint))
            })
            .collect()
    })
}
