use crate::{
    layout::Layout,
    model::{Candidate, Session, remember},
};
use smithay::utils::Rectangle;
fn candidates() -> Vec<Candidate<u8>> {
    [
        (1, "browser"),
        (2, "terminal"),
        (3, "browser"),
        (4, "editor"),
    ]
    .into_iter()
    .map(|(window, app)| Candidate {
        window,
        app: app.into(),
        title: app.into(),
    })
    .collect()
}
#[test]
fn groups_apps_by_recent_focus_and_confirms_the_most_recent_window() {
    let mut history = vec![1, 2, 4, 3];
    remember(&mut history, 3);
    let mut session = Session::new(candidates(), &history, false).unwrap();
    assert_eq!(
        session
            .apps
            .iter()
            .map(|a| a.id.as_str())
            .collect::<Vec<_>>(),
        ["browser", "terminal", "editor"]
    );
    assert_eq!(session.apps[0].windows, [3, 1]);
    assert_eq!(session.apps[0].title, "browser");
    assert_eq!(session.target(), Some(&2));
    session.step(true);
    assert_eq!(session.target(), Some(&3));
    session.step(true);
    assert_eq!(session.target(), Some(&4));
    session.step(false);
    assert_eq!(session.target(), Some(&3));
    assert_eq!(
        Session::new(candidates(), &history, true).unwrap().target(),
        Some(&4)
    );
}
#[test]
fn closed_windows_keep_selection_stable_or_choose_the_next_app() {
    let mut session = Session::new(candidates(), &[3, 1, 2, 4], false).unwrap();
    assert!(session.retain(|w| *w != 1));
    assert_eq!(session.target(), Some(&2));
    assert!(session.retain(|w| *w != 2));
    assert_eq!(session.target(), Some(&4));
    assert!(session.retain(|_| false));
    assert_eq!(session.target(), None);
    assert!(Session::<u8>::new(vec![], &[], false).is_none());
}
#[test]
fn overflowing_apps_keep_selection_visible_without_tiny_icons() {
    let area = Rectangle::new((-1200, 40).into(), (1200, 800).into());
    for selected in [0, 17, 39] {
        let layout = Layout::new(area, 40, selected).unwrap();
        assert!(layout.first <= selected && selected < layout.first + layout.visible);
        assert!(layout.icon >= 36);
        assert!(area.contains_rect(layout.rect));
        let point = (
            layout.rect.loc.x
                + 24
                + (selected - layout.first) as i32 * layout.slot
                + layout.slot / 2,
            layout.rect.loc.y + 50,
        );
        assert_eq!(
            layout.hit((f64::from(point.0), f64::from(point.1)).into()),
            Some(selected)
        );
        assert_eq!(layout.hit((0.0, 0.0).into()), None);
        assert_eq!(
            layout.hit((f64::from(point.0), f64::from(layout.rect.loc.y + 2)).into()),
            None
        );
    }
    assert!(Layout::new(area, 0, 0).is_none());
}
