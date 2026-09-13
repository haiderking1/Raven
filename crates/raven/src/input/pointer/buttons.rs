use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Deliver one seat-level press until every source holding that button releases.
#[derive(Debug)]
pub(crate) struct Buttons<S> {
    held: HashMap<u32, HashSet<S>>,
}
impl<S> Default for Buttons<S> {
    fn default() -> Self {
        Self {
            held: HashMap::new(),
        }
    }
}
impl<S: Eq + Hash + Clone> Buttons<S> {
    pub fn change(&mut self, source: S, button: u32, pressed: bool) -> bool {
        if pressed {
            let owners = self.held.entry(button).or_default();
            let first = owners.is_empty();
            owners.insert(source) && first
        } else {
            let Some(owners) = self.held.get_mut(&button) else {
                return false;
            };
            if !owners.remove(&source) || !owners.is_empty() {
                return false;
            }
            self.held.remove(&button);
            true
        }
    }
    pub fn remove(&mut self, source: &S) -> Vec<u32> {
        let mut released = Vec::new();
        self.held.retain(|button, owners| {
            owners.remove(source);
            if owners.is_empty() {
                released.push(*button);
                false
            } else {
                true
            }
        });
        released.sort_unstable();
        released
    }
    pub fn clear(&mut self) -> Vec<u32> {
        let mut released: Vec<_> = self.held.drain().map(|(button, _)| button).collect();
        released.sort_unstable();
        released
    }
}
