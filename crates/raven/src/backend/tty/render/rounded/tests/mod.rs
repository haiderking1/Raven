mod gpu;
mod scene;
use super::{cache::Cache, shape::Shape};
use smithay::{
    backend::renderer::{
        element::Id,
        utils::{CommitCounter, OpaqueRegions},
    },
    utils::Rectangle,
};
#[test]
fn stable_masks_keep_commits_and_corner_cutouts_are_not_opaque() {
    let mut cache = Cache::default();
    let id = Id::new();
    let mut source = CommitCounter::default();
    let bounds = Rectangle::new((10, 20).into(), (100, 80).into());
    let shape = Shape::new(bounds, 16.0);
    let first = cache.surface(id.clone(), source, shape);
    assert_eq!(first, cache.surface(id.clone(), source, shape));
    source.increment();
    let second = cache.surface(id.clone(), source, shape);
    assert_ne!(first, second);
    let moved = Shape::new(Rectangle::new((11, 20).into(), bounds.size), 16.0);
    assert_ne!(second, cache.surface(id, source, moved));
    let opaque = shape.opaque(
        OpaqueRegions::from_slice(&[Rectangle::from_size(bounds.size)]),
        bounds,
    );
    assert!(!opaque.iter().any(|r| r.contains((0, 0))));
    assert!(opaque.iter().any(|r| r.contains((50, 40))));
}
