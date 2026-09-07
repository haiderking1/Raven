use super::Wake;
use calloop::EventLoop;
use std::time::{Duration, Instant};

#[test]
fn deadline_source_rearms_earlier_fires_once_and_cancels_without_stale_wakeups() {
    let mut event_loop = EventLoop::try_new().unwrap();
    let mut received = Vec::new();
    let mut wake = Wake::new(
        event_loop.handle(),
        |deadline, received: &mut Vec<Instant>| received.push(deadline),
    )
    .unwrap();
    let now = Instant::now();
    wake.arm(Some(now + Duration::from_secs(60))).unwrap();
    wake.arm(Some(now)).unwrap();
    event_loop.dispatch(Duration::ZERO, &mut received).unwrap();
    assert_eq!(received, [now]);
    event_loop.dispatch(Duration::ZERO, &mut received).unwrap();
    assert_eq!(received, [now], "a one-shot timer must not repeat");

    // Rearm the same registration after it fired, then cancel it before dispatch.
    wake.arm(Some(now)).unwrap();
    wake.arm(None).unwrap();
    event_loop.dispatch(Duration::ZERO, &mut received).unwrap();
    assert_eq!(received, [now]);
    wake.arm(Some(now)).unwrap();
    event_loop.dispatch(Duration::ZERO, &mut received).unwrap();
    assert_eq!(received, [now, now]);

    wake.arm(Some(now)).unwrap();
    drop(wake);
    event_loop.dispatch(Duration::ZERO, &mut received).unwrap();
    assert_eq!(
        received,
        [now, now],
        "teardown must unregister the deadline"
    );
}
