use super::cache::Animations;
use std::collections::HashSet;

impl Animations {
    pub(super) fn retained_bytes(&self) -> usize {
        let mut seen = HashSet::new();
        self.entries
            .values()
            .map(|entry| {
                let mut bytes = entry.snapshot.retained_bytes(&mut seen);
                for image in entry
                    .last_queued_image
                    .iter()
                    .chain(entry.rendered_image.iter())
                {
                    bytes += image.old.retained_bytes(&mut seen);
                    bytes += image.current.retained_bytes(&mut seen);
                }
                bytes
            })
            .sum()
    }
}
