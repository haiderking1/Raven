use super::super::SceneElement;
use super::{
    accounting,
    blend::Blend,
    cache::{MAX_SNAPSHOT_BYTES, Transition},
    paint,
    snapshot::Snapshot,
};
use crate::{desktop::animation::timeline::Sample, state::State};
use smithay::{
    backend::renderer::{
        Texture,
        gles::{GlesRenderer, GlesTexProgram},
        utils::CommitCounter,
    },
    desktop::Window,
    utils::{Logical, Physical, Rectangle},
};
use std::{error::Error, rc::Rc};

fn scaled_opaque(
    image: &Snapshot,
    bounds: Rectangle<i32, Physical>,
) -> Vec<Rectangle<i32, Physical>> {
    let x = f64::from(bounds.size.w) / f64::from(image.texture.width());
    let y = f64::from(bounds.size.h) / f64::from(image.texture.height());
    image
        .opaque
        .iter()
        .filter_map(|rect| {
            // Round inward. Opaque metadata may be conservative, never hide pixels
            // exposed by fractional resampling at an opaque/transparent boundary.
            let inset_x = if x == 1.0 { 0.0 } else { x.max(1.0) };
            let inset_y = if y == 1.0 { 0.0 } else { y.max(1.0) };
            let lo = (
                (f64::from(rect.loc.x) * x + inset_x).ceil() as i32,
                (f64::from(rect.loc.y) * y + inset_y).ceil() as i32,
            );
            let hi = (
                (f64::from(rect.loc.x + rect.size.w) * x - inset_x).floor() as i32,
                (f64::from(rect.loc.y + rect.size.h) * y - inset_y).floor() as i32,
            );
            (hi.0 > lo.0 && hi.1 > lo.1).then(|| Rectangle::from_extremities(lo, hi))
        })
        .collect()
}

pub(super) fn build(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
    entry: &Transition,
    sample: Sample,
    commit: CommitCounter,
    program: Rc<GlesTexProgram>,
    available: usize,
) -> Result<Blend, Box<dyn Error>> {
    let snapshot = entry
        .snapshot
        .as_ref()
        .ok_or("live transition has no blend source")?;
    let target = super::content::bounds(state, window, area, scale)
        .ok_or("resize content has no visible window geometry")?;
    let bounds =
        super::content::interpolate(entry.content_from, target, f64::from(sample.progress));
    let mut elements = Vec::<SceneElement>::new();
    let (current, live) = if sample.progress > 0.0 {
        let bytes = (bounds.size.w as usize)
            .checked_mul(bounds.size.h as usize)
            .and_then(|n| n.checked_mul(4));
        if bounds.is_empty() || bytes.is_none_or(|bytes| bytes > available.min(MAX_SNAPSHOT_BYTES))
        {
            return Err("resize live-image memory limit exceeded".into());
        }
        if let Some(top) = window.toplevel() {
            super::tree::validate(renderer, top.wl_surface())?;
        }
        paint::append_live(
            renderer,
            state,
            window,
            area,
            scale,
            bounds,
            commit,
            &mut elements,
        );
        let live = accounting::live_regions(&elements, scale);
        (
            Rc::new(Snapshot::capture(renderer, bounds, scale, elements)?),
            live,
        )
    } else {
        (snapshot.clone(), Vec::new())
    };
    let old_opaque = scaled_opaque(snapshot, bounds);
    let current_opaque = scaled_opaque(&current, bounds);
    let opaque = if sample.progress <= 0.0 {
        old_opaque
    } else if sample.progress >= 1.0 {
        current_opaque
    } else {
        old_opaque
            .into_iter()
            .flat_map(|old| {
                current_opaque
                    .iter()
                    .filter_map(move |current| old.intersection(*current))
            })
            .collect()
    };
    Ok(Blend {
        id: entry.id.clone(),
        old: snapshot.clone(),
        current,
        bounds,
        progress: sample.progress,
        commit,
        program,
        live: live.into(),
        opaque: opaque.into(),
    })
}
