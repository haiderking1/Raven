use super::shape::Shape;
use smithay::backend::renderer::{element::Id, utils::CommitCounter};
use std::collections::HashMap;
#[derive(Default)]
pub(super) struct Cache {
    pub surfaces: HashMap<Id, Surface>,
    pub ring: Option<(Id, Shape, [f32; 4], Shape, CommitCounter)>,
}
pub(super) struct Surface {
    source: CommitCounter,
    shape: Shape,
    commit: CommitCounter,
    previous: Option<(CommitCounter, CommitCounter)>,
}
impl Cache {
    pub fn surface(
        &mut self,
        id: Id,
        source: CommitCounter,
        shape: Shape,
    ) -> (CommitCounter, Option<(CommitCounter, CommitCounter)>) {
        let entry = self.surfaces.entry(id).or_insert(Surface {
            source,
            shape,
            commit: CommitCounter::default(),
            previous: None,
        });
        if entry.source != source || entry.shape != shape {
            entry.previous = (entry.shape == shape).then_some((entry.commit, entry.source));
            entry.source = source;
            entry.shape = shape;
            entry.commit.increment();
        }
        (entry.commit, entry.previous)
    }
    pub fn ring(&mut self, shape: Shape, color: [f32; 4], inner: Shape) -> (Id, CommitCounter) {
        let entry = self
            .ring
            .get_or_insert_with(|| (Id::new(), shape, color, inner, CommitCounter::default()));
        if entry.1 != shape || entry.2 != color || entry.3 != inner {
            entry.1 = shape;
            entry.2 = color;
            entry.3 = inner;
            entry.4.increment();
        }
        (entry.0.clone(), entry.4)
    }
}
