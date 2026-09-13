use crate::buttons::Buttons;

#[test]
fn one_source_cannot_release_another_sources_button() {
    let mut buttons = Buttons::default();
    assert!(buttons.change("physical", 0x110, true));
    assert!(!buttons.change("virtual", 0x110, true));
    assert!(!buttons.change("virtual", 0x110, true));
    assert!(!buttons.change("virtual", 0x110, false));
    assert!(!buttons.change("virtual", 0x110, false));
    assert!(buttons.change("physical", 0x110, false));
    assert!(buttons.clear().is_empty());
}
#[test]
fn device_cleanup_releases_only_last_owned_buttons() {
    let mut buttons = Buttons::default();
    buttons.change(1, 0x110, true);
    buttons.change(2, 0x110, true);
    buttons.change(1, 0x111, true);
    assert_eq!(buttons.remove(&1), [0x111]);
    assert!(buttons.remove(&1).is_empty());
    assert_eq!(buttons.clear(), [0x110]);
    assert!(!buttons.change(2, 0x110, false));
}
