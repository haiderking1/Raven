use super::support::ConfigFile;
#[test]
fn screenshot_action_can_be_bound_to_print_and_a_compact_keyboard_chord() {
    let directory = ConfigFile::new();
    let result = directory
        .command(
            r#"raven.bind("Print", "screenshot")
raven.bind("Super+Shift+S", "screenshot")"#,
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
