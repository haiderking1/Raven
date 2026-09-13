mod frames;
mod paint;
mod prepared;
pub(crate) use prepared::PreparedCursor;
pub use settings::Settings as CursorSettings;
mod settings;
mod theme;

use frames::Animation;
use settings::Settings;
use smithay::input::pointer::{CursorIcon, CursorImageStatus};
use std::{collections::HashMap, io, rc::Rc, time::Instant};
use xcursor::CursorTheme;

pub(in crate::backend::tty) struct Cursors {
    theme: CursorTheme,
    size: u32,
    cache: HashMap<(CursorIcon, u32), Rc<Animation>>,
    fallback: Rc<Animation>,
    active: Option<(CursorIcon, u32, Instant)>,
    deadline: Option<Instant>,
}

impl Cursors {
    pub fn new() -> io::Result<Self> {
        Ok(Self::prepared(PreparedCursor::load(Settings::from_env()?)?))
    }

    pub fn prepared(prepared: PreparedCursor) -> Self {
        let fallback = Rc::new(prepared.fallback);
        let size = prepared.settings.size;
        let cache = HashMap::from([((CursorIcon::Default, size), fallback.clone())]);
        Self {
            theme: prepared.theme,
            size,
            cache,
            fallback,
            active: None,
            deadline: None,
        }
    }

    pub fn suspend(&mut self) {
        self.active = None;
        self.deadline = None;
    }

    /// Consume an expired wake even if DRM admission postpones rendering.
    /// The pending redraw then owns progress, avoiding repeated past-deadline wakes.
    pub fn tick(&mut self, status: &CursorImageStatus, now: Instant) -> bool {
        if !matches!((status, self.active), (CursorImageStatus::Named(icon), Some((active, _, _))) if *icon == active)
        {
            self.suspend();
        }
        if self.deadline.is_some_and(|deadline| deadline <= now) {
            self.deadline = None;
            true
        } else {
            false
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    fn animation(&mut self, icon: CursorIcon, target: u32) -> Rc<Animation> {
        if let Some(animation) = self.cache.get(&(icon, target)) {
            return animation.clone();
        }
        let animation = match theme::load(&self.theme, icon, target, self.size) {
            Ok(animation) => Rc::new(animation),
            Err(error) => {
                eprintln!(
                    "raven: cannot load cursor {} at {target}px: {error}; using theme default",
                    icon.name()
                );
                if icon == CursorIcon::Default {
                    self.fallback.clone()
                } else {
                    self.animation(CursorIcon::Default, target)
                }
            }
        };
        self.cache.insert((icon, target), animation.clone());
        animation
    }
}
