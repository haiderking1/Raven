mod driver;

use super::{CAPACITY, Pool, slot::State};
use driver::Driver;
use std::time::{Duration, Instant};

#[test]
fn pending_pool_is_bounded_and_only_ready_names_are_reused() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    for _ in 0..CAPACITY + 1 {
        pool.begin(&gl);
        pool.end(&gl);
    }
    assert_eq!(gl.begins.get(), CAPACITY);
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(gl.polls.get(), CAPACITY);
    assert_eq!(gl.reads.get(), 0);

    gl.ready.set([true, true, false, false]);
    assert_eq!(pool.sample(&gl), Some(Duration::from_nanos(40)));
    assert_eq!(gl.reads.get(), 2);
    pool.begin(&gl);
    assert_eq!(gl.current.get(), 1);
    pool.end(&gl);
    // Recycled slot 1 is newer than pending slot 3. Slot order is no
    // longer submission order, so polling must not stop at slot 1.
    gl.ready.set([false, true, true, false]);
    assert_eq!(pool.sample(&gl), Some(Duration::from_nanos(20)));
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(gl.reads.get(), 3, "samples must be consumed only once");
}

#[test]
fn disjoint_during_collection_invalidates_pending_and_open_intervals() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    pool.begin(&gl);
    pool.end(&gl);
    pool.begin(&gl);
    gl.ready.set([true, false, false, false]);
    gl.disjoint_on_result.set(true);
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(gl.current.get(), 2, "sampling must not end an open bracket");
    pool.end(&gl);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(
        gl.reads.get(),
        1,
        "invalid queries need no result retrieval"
    );
    pool.begin(&gl);
    pool.end(&gl);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), Some(Duration::from_nanos(10)));
}

#[test]
fn reset_closes_and_discards_without_recycling_in_flight_queries() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    pool.begin(&gl);
    pool.reset(&gl);
    assert_eq!(gl.current.get(), 0);
    pool.begin(&gl);
    assert_eq!(gl.current.get(), 2);
    pool.end(&gl);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), Some(Duration::from_nanos(40)));
    assert_eq!(gl.reads.get(), 1);
    pool.begin(&gl);
    pool.destroy(&gl);
    assert_eq!(&*gl.deleted.borrow(), &[1, 2, 3, 4]);
    assert_eq!(
        gl.reads.get(),
        1,
        "destroy must not wait for pending results"
    );
    assert!(!pool.enabled());
}

#[test]
fn foreign_queries_are_neither_overlapped_nor_ended() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    gl.current.set(99);
    pool.begin(&gl);
    pool.end(&gl);
    assert_eq!(gl.current.get(), 99);
    assert_eq!(gl.begins.get(), 0);
    assert_eq!(gl.ends.get(), 0);
    gl.current.set(0);
    pool.begin(&gl);
    gl.current.set(99); // Simulate a caller replacing our active query.
    pool.end(&gl);
    assert_eq!(gl.current.get(), 99);
    assert_eq!(gl.ends.get(), 0);
    assert!(!pool.enabled());
}

#[test]
fn nested_brackets_preserve_ownership_but_discard_ambiguous_timing() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    pool.begin(&gl);
    pool.begin(&gl);
    pool.end(&gl);
    assert_eq!(gl.current.get(), 1);
    pool.end(&gl);
    pool.end(&gl);
    assert_eq!(gl.ends.get(), 1);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(gl.reads.get(), 0);
}

#[test]
fn reset_during_result_read_disables_and_abandons_context_local_names() {
    let gl = Driver::default();
    let mut pool = Pool::new(&gl).unwrap();
    pool.begin(&gl);
    pool.end(&gl);
    gl.ready.set([true; CAPACITY]);
    gl.loss_on_result.set(true);
    assert_eq!(pool.sample(&gl), None);
    assert!(!pool.enabled());
    let begins = gl.begins.get();
    pool.reset(&gl);
    pool.begin(&gl);
    pool.end(&gl);
    pool.destroy(&gl);
    assert_eq!(gl.begins.get(), begins);
    assert!(
        gl.deleted.borrow().is_empty(),
        "never delete lost-context names elsewhere"
    );
}

#[test]
fn counter_widths_and_possible_overflow_are_rejected() {
    for bits in [0, 29, 65] {
        assert!(
            Pool::new(&Driver {
                bits,
                ..Driver::default()
            })
            .is_none()
        );
    }
    let gl = Driver {
        bits: 30,
        ..Driver::default()
    };
    let mut pool = Pool::new(&gl).unwrap();
    pool.begin(&gl);
    pool.end(&gl);
    let State::Pending(ref mut interval) = pool.slots[0].state else {
        unreachable!()
    };
    interval.start = Instant::now() - Duration::from_secs(2);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), None);
    assert_eq!(gl.reads.get(), 0, "possibly wrapped results are unusable");
    pool.begin(&gl);
    pool.end(&gl);
    gl.results.set([(1 << 30) - 1; CAPACITY]);
    gl.ready.set([true; CAPACITY]);
    assert_eq!(pool.sample(&gl), None, "saturated values are not timings");
}
