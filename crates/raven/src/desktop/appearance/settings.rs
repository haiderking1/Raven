/// Largest accepted logical-pixel gap or border. Geometry clamps further to fit.
pub const MAX_EXTENT: i32 = 65_535;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InnerGaps {
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OuterGaps {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

/// Colors use straight RGBA in 0..=1. Rendering premultiplies RGB by alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub width: i32,
    pub active: [f32; 4],
    pub inactive: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Appearance {
    pub inner: InnerGaps,
    pub outer: OuterGaps,
    pub border: Border,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidAppearance(pub &'static str);

impl std::fmt::Display for InvalidAppearance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for InvalidAppearance {}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            inner: InnerGaps {
                horizontal: 8,
                vertical: 8,
            },
            outer: OuterGaps {
                top: 8,
                right: 8,
                bottom: 8,
                left: 8,
            },
            border: Border {
                width: 2,
                active: [0.36, 0.48, 0.58, 1.0],
                inactive: [0.20, 0.22, 0.25, 1.0],
            },
        }
    }
}

impl Appearance {
    /// Explicit edge-to-edge layout for user opt-out and legacy fixtures.
    pub fn disabled() -> Self {
        Self {
            inner: InnerGaps::default(),
            outer: OuterGaps::default(),
            border: Border {
                width: 0,
                ..Self::default().border
            },
        }
    }

    pub fn validate(&self) -> Result<(), InvalidAppearance> {
        for extent in [
            self.inner.horizontal,
            self.inner.vertical,
            self.outer.top,
            self.outer.right,
            self.outer.bottom,
            self.outer.left,
            self.border.width,
        ] {
            if !(0..=MAX_EXTENT).contains(&extent) {
                return Err(InvalidAppearance(
                    "gaps and border width must be in 0..=65535",
                ));
            }
        }
        for channel in self.border.active.into_iter().chain(self.border.inactive) {
            if !channel.is_finite() || !(0.0..=1.0).contains(&channel) {
                return Err(InvalidAppearance(
                    "RGBA channels must be finite and in 0..=1",
                ));
            }
        }
        Ok(())
    }
}

impl Border {
    pub(crate) fn premultiplied(&self, active: bool) -> [f32; 4] {
        let [r, g, b, a] = if active { self.active } else { self.inactive };
        [r * a, g * a, b * a, a]
    }
}
