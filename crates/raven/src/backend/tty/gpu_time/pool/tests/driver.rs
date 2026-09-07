use super::super::{CAPACITY, Queries};
use std::cell::{Cell, RefCell};

pub(super) struct Driver {
    pub bits: i32,
    pub current: Cell<u32>,
    pub ready: Cell<[bool; CAPACITY]>,
    pub results: Cell<[u64; CAPACITY]>,
    pub disjoint: Cell<bool>,
    pub disjoint_on_result: Cell<bool>,
    pub lost: Cell<bool>,
    pub loss_on_result: Cell<bool>,
    pub reads: Cell<usize>,
    pub polls: Cell<usize>,
    pub begins: Cell<usize>,
    pub ends: Cell<usize>,
    pub deleted: RefCell<Vec<u32>>,
}

impl Default for Driver {
    fn default() -> Self {
        Self {
            bits: 64,
            current: Cell::new(0),
            ready: Cell::new([false; CAPACITY]),
            results: Cell::new([10, 40, 20, 30]),
            disjoint: Cell::new(false),
            disjoint_on_result: Cell::new(false),
            lost: Cell::new(false),
            loss_on_result: Cell::new(false),
            reads: Cell::new(0),
            polls: Cell::new(0),
            begins: Cell::new(0),
            ends: Cell::new(0),
            deleted: RefCell::new(Vec::new()),
        }
    }
}

impl Queries for Driver {
    fn reset_detected(&self) -> bool {
        self.lost.get()
    }
    fn disjoint(&self) -> bool {
        self.disjoint.replace(false)
    }
    fn counter_bits(&self) -> i32 {
        self.bits
    }
    fn current(&self) -> u32 {
        self.current.get()
    }
    fn generate(&self, ids: &mut [u32]) {
        for (index, id) in ids.iter_mut().enumerate() {
            *id = index as u32 + 1;
        }
    }
    fn delete(&self, ids: &[u32]) {
        assert_eq!(
            self.current.get(),
            0,
            "cleanup must close our active query first"
        );
        self.deleted.borrow_mut().extend_from_slice(ids);
    }
    fn begin(&self, id: u32) {
        assert_eq!(self.current.replace(id), 0, "queries must never overlap");
        let mut ready = self.ready.get();
        ready[id as usize - 1] = false;
        self.ready.set(ready);
        self.begins.set(self.begins.get() + 1);
    }
    fn end(&self) {
        assert_ne!(self.current.replace(0), 0);
        self.ends.set(self.ends.get() + 1);
    }
    fn available(&self, id: u32) -> bool {
        assert_ne!(self.current.get(), id, "cannot poll an active query");
        self.polls.set(self.polls.get() + 1);
        self.ready.get()[id as usize - 1]
    }
    fn result(&self, id: u32) -> u64 {
        assert!(
            self.ready.get()[id as usize - 1],
            "result retrieval would block"
        );
        self.reads.set(self.reads.get() + 1);
        if self.disjoint_on_result.replace(false) {
            self.disjoint.set(true);
        }
        if self.loss_on_result.replace(false) {
            self.lost.set(true);
        }
        self.results.get()[id as usize - 1]
    }
}
