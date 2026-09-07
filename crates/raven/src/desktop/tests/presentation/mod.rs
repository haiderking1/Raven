mod queued;

use super::fixture::Fixture;

#[test]
fn replacing_unpresented_content_without_new_feedback_discards_the_old_request() {
    let mut f = Fixture::new();
    let presentation = super::globals::bind(&mut f, "wp_presentation", 1);
    let top = f.toplevel();
    f.configure(top);
    let feedback = f.id();
    f.wire.request(presentation, 1, &[top.surface, feedback]);
    let first = f.buffer();
    f.attach(top, first);
    let second = f.buffer();
    f.wire.request(top.surface, 1, &[second, 0, 0]);
    f.wire.request(top.surface, 6, &[]);
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|event| event.object == feedback && event.opcode == 2),
        "superseded content kept feedback that could be attributed to another buffer"
    );
}
