/// Extend a valid 32-bit DRM sequence across wraps. A backwards reset loses
/// the hardware epoch; report protocol-defined unknown (zero) after that.
#[derive(Default)]
pub(super) struct Sequence {
    previous: Option<(u32, u64)>,
    unknown: bool,
}

impl Sequence {
    pub fn observe(&mut self, raw: Option<u32>) -> u64 {
        let Some(raw) = raw else {
            return 0;
        };
        if self.unknown {
            return 0;
        }
        let extended = if let Some((previous, total)) = self.previous {
            let step = raw.wrapping_sub(previous);
            if step > u32::MAX / 2 {
                self.unknown = true;
                return 0;
            }
            let Some(total) = total.checked_add(u64::from(step)) else {
                self.unknown = true;
                return 0;
            };
            total
        } else {
            u64::from(raw)
        };
        self.previous = Some((raw, extended));
        extended
    }
}
