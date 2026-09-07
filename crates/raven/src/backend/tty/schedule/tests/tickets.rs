use super::super::Schedule;
use super::{Duration, Instant, Policy};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
struct Ticket {
    id: u8,
    dropped: Rc<RefCell<Vec<u8>>>,
}

impl Ticket {
    fn new(id: u8, dropped: &Rc<RefCell<Vec<u8>>>) -> Self {
        Self {
            id,
            dropped: Rc::clone(dropped),
        }
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        self.dropped.borrow_mut().push(self.id);
    }
}

#[test]
fn full_capacity_never_replaces_tickets_and_all_reset_paths_drop_both() {
    for reset in ["pause", "resume", "drop"] {
        let now = Instant::now();
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let mut schedule = Schedule::with_policy(50_000, now, Policy::Deadline);
        assert!(schedule.pending().is_none());
        assert!(schedule.queued().is_none());
        schedule.rendered(Some(Ticket::new(1, &dropped)), now);
        schedule.request_redraw();
        let window = now + Duration::from_micros(17_500);
        schedule.rendered(Some(Ticket::new(2, &dropped)), window);
        schedule.request_redraw();
        // A caller violating render_due cannot grow or overwrite either slot.
        schedule.rendered(Some(Ticket::new(3, &dropped)), window);
        assert_eq!(schedule.pending().unwrap().id, 1);
        assert_eq!(schedule.queued().unwrap().id, 2);
        assert_eq!(*dropped.borrow(), vec![3]);
        assert!(!schedule.render_due(window));

        match reset {
            "pause" => {
                schedule.pause();
                assert!(!schedule.active());
                assert!(!schedule.render_due(window));
                assert!(!schedule.callbacks_due(window));
            }
            "resume" => {
                schedule.resume(window);
                assert!(schedule.active());
                assert!(schedule.render_due(window));
                assert!(!schedule.callbacks_due(window));
            }
            "drop" => {}
            _ => unreachable!(),
        }
        if reset != "drop" {
            assert!(schedule.pending().is_none());
            assert!(schedule.queued().is_none());
            assert_eq!(schedule.deadline(), None);
            assert_eq!(dropped.borrow().len(), 3);
        }
        drop(schedule);
        let mut ids = dropped.borrow().clone();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}

#[test]
fn oldest_watchdog_survives_queue_and_promotion_uses_actual_submit_time() {
    let now = Instant::now();
    let dropped = Rc::new(RefCell::new(Vec::new()));
    let mut schedule = Schedule::with_policy(50_000, now, Policy::Deadline);
    schedule.rendered(Some(Ticket::new(1, &dropped)), now);
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    let window = now + Duration::from_micros(17_500);
    schedule.rendered(Some(Ticket::new(2, &dropped)), window);
    let oldest_timeout = now + Duration::from_secs(3);
    assert_eq!(schedule.deadline(), Some(oldest_timeout));
    assert!(!schedule.stalled(oldest_timeout - Duration::from_nanos(1)));
    assert!(schedule.stalled(oldest_timeout));

    let submit = now + Duration::from_secs(1);
    let completed = schedule
        .presented(now + Duration::from_millis(20), submit)
        .unwrap();
    assert_eq!(completed.id, 1);
    assert!(dropped.borrow().is_empty());
    assert_eq!(schedule.pending().unwrap().id, 2);
    assert!(schedule.queued().is_none());
    let promoted_timeout = submit + Duration::from_secs(3);
    assert_eq!(schedule.deadline(), Some(promoted_timeout));
    assert!(!schedule.stalled(oldest_timeout));
    assert!(!schedule.stalled(promoted_timeout - Duration::from_nanos(1)));
    assert!(schedule.stalled(promoted_timeout));
    drop(completed);
    assert_eq!(*dropped.borrow(), vec![1]);

    // Deferred KMS submission fails immediately. Ownership returns, not drops.
    let failed = schedule.discard_pending().unwrap();
    assert_eq!(failed.id, 2);
    assert_eq!(*dropped.borrow(), vec![1]);
    assert!(schedule.pending().is_none());
    assert!(schedule.render_due(submit));
    assert!(!schedule.stalled(promoted_timeout));
    assert_eq!(schedule.deadline(), None);
    assert!(schedule.discard_pending().is_none());
    drop(failed);
    assert_eq!(*dropped.borrow(), vec![1, 2]);
}
