use super::{Duration, Instant, Schedule};

#[test]
fn empty_redraws_keep_callback_cadence_without_spinning() {
    let now = Instant::now();
    let mut schedule = Schedule::new(50_000, now);
    let due = now + Duration::from_millis(20);
    for elapsed_ms in [3, 5, 15] {
        let render = now + Duration::from_millis(elapsed_ms);
        schedule.request_redraw();
        assert!(schedule.render_due(render));
        schedule.rendered(None, render);
        assert_eq!(schedule.deadline(), Some(due));
        // An empty render may revisit this cycle, but cannot advance it.
        assert!(schedule.callbacks_due(render));
        assert!(!schedule.callbacks_advance_cycle(render));
        schedule.callbacks_sent(render);
        assert!(!schedule.callbacks_due(render));
        assert!(!schedule.render_due(render));
    }
    assert!(!schedule.callbacks_due(due - Duration::from_nanos(1)));
    let sent = now + Duration::from_millis(25);
    assert!(schedule.callbacks_due(sent));
    assert!(schedule.callbacks_advance_cycle(sent));
    schedule.callbacks_sent(sent);
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.callbacks_due(sent + Duration::from_secs(10)));

    schedule.request_redraw();
    schedule.rendered(None, sent + Duration::from_millis(2));
    // The late wake did not move the display grid from 40 ms to 45 ms.
    assert_eq!(schedule.deadline(), Some(now + Duration::from_millis(40)));
    schedule.callbacks_sent(sent + Duration::from_millis(2));
    assert!(!schedule.callbacks_due(now + Duration::from_millis(39)));
    assert!(schedule.callbacks_advance_cycle(now + Duration::from_millis(40)));
}

#[test]
fn empty_render_preserves_callbacks_already_due_from_presentation() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    schedule.rendered(Some(()), now);
    assert!(schedule.callbacks_advance_cycle(now));
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    let flip = now + Duration::from_millis(5);
    assert_eq!(schedule.presented(flip, flip), Some(()));
    let render = flip + Duration::from_millis(1);
    schedule.rendered(None, render);
    assert_eq!(
        schedule.deadline(),
        Some(flip + Duration::from_nanos(16_666_666))
    );
    assert!(schedule.callbacks_due(render));
    assert!(!schedule.callbacks_advance_cycle(render));
    assert!(!schedule.render_due(render));

    // Another empty render in the expired timer's batch must preserve NewCycle
    // even though it arms the following refresh before callbacks are delivered.
    let due = flip + Duration::from_nanos(16_666_666);
    schedule.request_redraw();
    schedule.rendered(None, due);
    assert!(schedule.callbacks_advance_cycle(due));
    assert_eq!(
        schedule.deadline(),
        Some(due + Duration::from_nanos(16_666_666))
    );
    schedule.callbacks_sent(due);
    assert!(!schedule.callbacks_due(due));
}

#[test]
fn same_batch_pageflip_preserves_unsent_submission_cycle() {
    for policy in [super::Policy::Immediate, super::Policy::Deadline] {
        let now = Instant::now();
        let mut schedule = Schedule::with_policy(50_000, now, policy);
        schedule.rendered(Some(()), now);
        let flip = now + Duration::from_millis(20);
        assert_eq!(schedule.presented(flip, flip), Some(()));
        assert!(schedule.callbacks_advance_cycle(flip));
        schedule.callbacks_sent(flip);
        assert!(!schedule.callbacks_due(flip));
    }
}

#[test]
fn successor_callbacks_wait_for_submission_and_survive_another_same_batch_flip() {
    let now = Instant::now();
    let mut schedule = Schedule::with_policy(50_000, now, super::Policy::Deadline);
    schedule.rendered(Some(()), now);
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    let window = now + Duration::from_micros(17_500);
    schedule.rendered(Some(()), window);
    assert!(!schedule.callbacks_due(window));
    let first = now + Duration::from_millis(20);
    assert_eq!(schedule.presented(first, first), Some(()));
    assert!(schedule.callbacks_advance_cycle(first));
    let second = now + Duration::from_millis(40);
    assert_eq!(schedule.presented(second, second), Some(()));
    assert!(schedule.callbacks_advance_cycle(second));
    schedule.callbacks_sent(second);
    assert!(!schedule.callbacks_due(second));
}
