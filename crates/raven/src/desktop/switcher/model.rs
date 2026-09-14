//! Window-independent selection rules; no focus changes during preview.
#[derive(Clone, Debug)]
pub(crate) struct Candidate<W> {
    pub window: W,
    pub app: String,
    pub title: String,
}
#[derive(Clone, Debug)]
pub(crate) struct App<W> {
    pub id: String,
    pub title: String,
    pub windows: Vec<W>,
}
#[derive(Debug)]
pub(crate) struct Session<W> {
    pub apps: Vec<App<W>>,
    pub selected: usize,
}
impl<W: Clone + Eq> Session<W> {
    pub fn new(candidates: Vec<Candidate<W>>, history: &[W], reverse: bool) -> Option<Self> {
        let mut candidates = candidates;
        candidates.sort_by_key(|c| {
            history
                .iter()
                .position(|w| w == &c.window)
                .unwrap_or(usize::MAX)
        });
        let mut apps: Vec<App<W>> = Vec::new();
        for candidate in candidates {
            if let Some(app) = apps.iter_mut().find(|app| app.id == candidate.app) {
                app.windows.push(candidate.window);
            } else {
                apps.push(App {
                    id: candidate.app,
                    title: candidate.title,
                    windows: vec![candidate.window],
                });
            }
        }
        if apps.is_empty() {
            return None;
        }
        let selected = if reverse {
            apps.len() - 1
        } else {
            1 % apps.len()
        };
        Some(Self { apps, selected })
    }
    pub fn step(&mut self, reverse: bool) {
        if self.apps.is_empty() {
            return;
        }
        self.selected = if reverse {
            (self.selected + self.apps.len() - 1) % self.apps.len()
        } else {
            (self.selected + 1) % self.apps.len()
        };
    }
    pub fn target(&self) -> Option<&W> {
        self.apps.get(self.selected)?.windows.first()
    }
    pub fn retain(&mut self, mut alive: impl FnMut(&W) -> bool) -> bool {
        let selected = self.apps.get(self.selected).map(|app| app.id.clone());
        let before: usize = self.apps.iter().map(|app| app.windows.len()).sum();
        for app in &mut self.apps {
            app.windows.retain(&mut alive);
        }
        self.apps.retain(|app| !app.windows.is_empty());
        self.selected = selected
            .and_then(|id| self.apps.iter().position(|app| app.id == id))
            .unwrap_or(self.selected.min(self.apps.len().saturating_sub(1)));
        before != self.apps.iter().map(|app| app.windows.len()).sum()
    }
}
pub(crate) fn remember<W: Eq>(history: &mut Vec<W>, window: W) {
    history.retain(|candidate| candidate != &window);
    history.insert(0, window);
}
